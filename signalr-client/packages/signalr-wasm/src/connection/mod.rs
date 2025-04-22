mod handshake;
mod reader;
mod receive_invocation;
mod send_invocation;

use futures::channel::mpsc::{self, Receiver, Sender};
use futures::SinkExt;
use serde::Serialize;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::MessageEvent;

use wasm_bindgen::prelude::*;

use crate::message::{CompletionMessage, InvocationMessage};
use wasm_bindgen::closure::Closure;
use web_sys::WebSocket;

const CHANNEL_BOUND_SIZE: usize = 64;

type CompletionSubscriberMap = HashMap<String, Sender<CompletionMessage>>;
type InvocationSubscriberMap = HashMap<String, Sender<InvocationMessage>>;

pub struct SignalRConnection {
    url: String,
    web_socket: OnceCell<WebSocket>,
    on_message_closure: Option<Closure<dyn FnMut(MessageEvent)>>,
    invocation_id: u64,
    completion_subscribers: Rc<RefCell<CompletionSubscriberMap>>,
    invocation_subscribers: Rc<RefCell<InvocationSubscriberMap>>,
}

impl SignalRConnection {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
            web_socket: OnceCell::new(),
            on_message_closure: None,
            invocation_id: 0,
            completion_subscribers: Rc::new(RefCell::new(CompletionSubscriberMap::new())),
            invocation_subscribers: Rc::new(RefCell::new(InvocationSubscriberMap::new())),
        }
    }

    fn parse_message(e: &MessageEvent) -> Result<Vec<String>, String> {
        let data: String;

        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            data = text.into();
        } else {
            return Err("Unsupported wire format".to_owned());
        }

        Ok(data.split_terminator('\x1E').map(str::to_owned).collect())
    }

    fn open_message_channel(&mut self) -> Result<Receiver<String>, String> {
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
                    Ok(vec) => vec,
                    Err(e) => {
                        console_error!("Failed to parse message: {}", e);
                        return;
                    }
                };

                for message in parsed {
                    if let Err(e) = sender_clone.send(message).await {
                        console_error!("Failed to send message: {}", e);
                        return;
                    }
                }
            });
        });

        ws.set_onmessage(Some(on_message_closure.as_ref().unchecked_ref()));
        self.on_message_closure = Some(on_message_closure);

        Ok(receiver)
    }

    fn send_struct(ws: &WebSocket, message: &impl Serialize) -> Result<(), String> {
        let serialized = serde_json::to_string(message)
            .map(|mut s| {
                s.push('\x1E');
                s
            })
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        ws.send_with_str(serialized.as_str())
            .map_err(|e| format!("Failed to send message: {:?}", e))
    }
}

impl Drop for SignalRConnection {
    fn drop(&mut self) {
        if let Some(ws) = self.web_socket.get_mut() {
            ws.set_onmessage(None);
        }
    }
}
