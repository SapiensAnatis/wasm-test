use futures::channel::mpsc::{self, Receiver};
use futures::channel::oneshot;
use futures::SinkExt;
use serde::Deserialize;
use std::cell::OnceCell;
use wasm_bindgen_futures::spawn_local;
use web_sys::MessageEvent;

use wasm_bindgen::prelude::*;

use wasm_bindgen::closure::Closure;
use web_sys::WebSocket;

const CHANNEL_BOUND_SIZE: usize = 64;

#[derive(Debug)]
pub enum WebSocketEvent {
    Open,
    Message(Vec<u8>),
}

impl Clone for WebSocketEvent {
    fn clone(&self) -> Self {
        match self {
            WebSocketEvent::Open => WebSocketEvent::Open,
            WebSocketEvent::Message(vec) => WebSocketEvent::Message(vec.clone()),
        }
    }
}

pub struct SignalRConnection {
    url: String,
    web_socket: OnceCell<WebSocket>,
    on_message_closure: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl SignalRConnection {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
            web_socket: OnceCell::new(),
            on_message_closure: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err(String::from("Failed to create websocket"));
            }
        };

        let (open_sender, open_receiver) = oneshot::channel::<()>();
        let (handshake_sender, handshake_receiver) = oneshot::channel::<Result<(), String>>();

        let on_open = Closure::once(move || {
            if let Err(e) = open_sender.send(()) {
                console_error!("Failed to send open message: {:?}", e);
            }
        });

        let on_message = Closure::once(move |e: MessageEvent| {
            let parse_result = Self::parse_message(&e);

            /* TODO: improve error handling here - can we avoid expect / panics? */

            let message: String = match parse_result {
                Ok(msg) => msg,
                Err(e) => {
                    handshake_sender
                        .send(Err(e))
                        .expect("Failed to send on_message error");
                    return;
                }
            };

            console_log!("Received handshake response: {}", message);

            let parsed_result: HandshakeResponse = match serde_json::from_str(&message) {
                Ok(val) => val,
                Err(e) => {
                    handshake_sender
                        .send(Err(format!("Failed to parse JSON: {}", e)))
                        .expect("Failed to send on_message error");
                    return;
                }
            };

            if let Some(error) = parsed_result.error {
                handshake_sender
                    .send(Err(format!("Received handshake error: {}", error)))
                    .expect("Failed to send on_message error");
                return;
            }

            handshake_sender
                .send(Ok(()))
                .expect("Failed to send on_message success");
        });

        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        match open_receiver.await {
            Ok(()) => {
                console_log!("Received open event, transmitting handshake...");
            }
            Err(e) => return Err(format!("Failed to get open event: {}", e)),
        }

        if let Err(e) = ws.send_with_str("{\"protocol\": \"json\", \"version\": 1}\x1E") {
            return Err(format!("Failed to send handshake: {:?}", e));
        }

        match handshake_receiver.await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("Handshake failed: {}", e));
                }

                console_log!("Successfully established connection");
            }
            Err(e) => return Err(format!("Failed to get handshake event: {}", e)),
        }

        ws.set_onopen(None);
        ws.set_onmessage(None);

        // TODO: handle gracefully
        self.web_socket.set(ws).unwrap();

        Ok(())
    }

    fn parse_message(e: &MessageEvent) -> Result<String, String> {
        let data: String;

        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            data = text.into();
        } else {
            return Err("Unsupported wire format".to_owned());
        }

        if let Some(str) = data.strip_suffix('\x1E') {
            Ok(str.to_owned())
        } else {
            Err("Unexpected message format".to_owned())
        }
    }

    pub fn open_message_channel(&mut self) -> Result<Receiver<String>, String> {
        if self.on_message_closure.is_some() {
            return Err("Already listening for messages".to_owned());
        }

        let ws: &WebSocket = match self.web_socket.get_mut() {
            Some(ws) => ws,
            None => {
                return Err("No open socket".to_owned());
            }
        };

        let (sender, receiver) = mpsc::channel::<String>(CHANNEL_BOUND_SIZE);

        let on_message_closure = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let mut sender_clone = sender.clone();

            spawn_local(async move {
                let parsed = match Self::parse_message(&e) {
                    Ok(str) => str,
                    Err(e) => {
                        console_error!("Failed to parse message: {}", e);
                        return;
                    }
                };

                if let Err(e) = sender_clone.send(parsed).await {
                    console_error!("Failed to send message: {}", e);
                    return;
                }
            });
        });

        ws.set_onmessage(Some(on_message_closure.as_ref().unchecked_ref()));
        self.on_message_closure = Some(on_message_closure);

        Ok(receiver)
    }
}

impl Drop for SignalRConnection {
    fn drop(&mut self) {
        if let Some(ws) = self.web_socket.get_mut() {
            ws.set_onmessage(None);
        }
    }
}

#[derive(Deserialize)]
struct HandshakeResponse {
    error: Option<String>,
}
