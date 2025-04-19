use async_broadcast::{broadcast, Receiver, Sender};
use futures::StreamExt;
use std::{cell::OnceCell, fmt};
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
    pub event_receiver: Receiver<WebSocketEvent>,
    event_sender: Sender<WebSocketEvent>,
    web_socket: OnceCell<WebSocket>,
    on_open: OnceCell<Closure<dyn FnMut() -> ()>>,
    on_message: OnceCell<Closure<dyn FnMut(MessageEvent) -> ()>>,
}

impl SignalRConnection {
    pub fn new(url: &str) -> Self {
        let (event_sender, event_receiver) = broadcast::<WebSocketEvent>(CHANNEL_BOUND_SIZE);

        Self {
            url: String::from(url),
            event_receiver,
            event_sender,
            web_socket: OnceCell::new(),
            on_open: OnceCell::new(),
            on_message: OnceCell::new(),
        }
    }

    pub async fn connect(self: &mut Self) -> Result<(), String> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err(String::from("Failed to create websocket"));
            }
        };

        let open_event_sender = self.event_sender.clone();
        let message_event_sender = self.event_sender.clone();

        let on_open = self.on_open.get_or_init(|| {
            Closure::<dyn FnMut()>::new(move || Self::on_open(open_event_sender.clone()))
        });

        let on_message = self.on_message.get_or_init(|| {
            Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                Self::on_message(message_event_sender.clone(), e)
            })
        });

        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        match self.event_receiver.next().await {
            Some(WebSocketEvent::Open) => {
                console_log!("Connected");
            }
            _ => {
                return Err("Failed to get connected event".into());
            }
        }

        if let Err(e) = ws.send_with_str("{\"protocol\": \"json\", \"version\": 1}\x1E") {
            return Err(format!("Failed to send handshake: {:?}", e));
        }

        match self.event_receiver.next().await {
            Some(WebSocketEvent::Message(data)) => {
                let string = str::from_utf8(data.as_slice()).unwrap_or("FAILED TO LOAD BYTES");
                console_log!("Message received: {}", string);
            }
            _ => {
                return Err("Failed to get first message event".into());
            }
        }

        // TODO: handle gracefully
        self.web_socket.set(ws).unwrap();

        Ok(())
    }

    fn on_open(sender: Sender<WebSocketEvent>) {
        spawn_local(async move {
            match sender.broadcast(WebSocketEvent::Open).await {
                Ok(_) => {}
                Err(e) => console_error!("Failed to send open event: {}", e),
            }
        })
    }

    fn on_message(sender: Sender<WebSocketEvent>, e: MessageEvent) {
        spawn_local(async move {
            let data: String;

            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                data = text.into();
            } else {
                unimplemented!("Unsupported wire format");
            }

            match sender
                .broadcast(WebSocketEvent::Message(data.into_bytes()))
                .await
            {
                Ok(_) => {}
                Err(e) => console_error!("Failed to send message event: {}", e),
            }
        })
    }
}

#[derive(Copy, Clone)]
enum HandshakeStatus {
    WaitingToConnect,
    NotStarted,
    InProgress,
    Complete,
    Error,
}

impl fmt::Display for HandshakeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandshakeStatus::WaitingToConnect => write!(f, "WAITING_TO_CONNECT"),
            HandshakeStatus::NotStarted => write!(f, "NOT_STARTED"),
            HandshakeStatus::InProgress => write!(f, "IN_PROGRESS"),
            HandshakeStatus::Complete => write!(f, "COMPLETE"),
            HandshakeStatus::Error => write!(f, "ERROR"),
        }
    }
}
