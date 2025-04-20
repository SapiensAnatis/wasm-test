use std::{cell::OnceCell, collections::HashMap, rc::Rc};

use futures::{
    channel::oneshot::{self, Receiver},
    StreamExt,
};
use serde::Deserialize;
use serde_json::Value;
use signalr_wasm::connection::SignalRConnection;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

type OneshotSender<T> = futures::channel::oneshot::Sender<T>;

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
    invocation_subscribers: Rc<RefCell<HashMap<String, OneshotSender<String>>>>,
    invocation_id: usize,
}

impl ChatClient {
    pub fn start_reader(&mut self) -> Result<(), String> {
        console_log!("Starting read loop");

        let mut receiver = self.connection.open_message_channel()?;
        let subscribers_clone = self.invocation_subscribers.clone();

        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                console_log!("Received message: {}", message);

                let parsed_message: Value = match serde_json::from_str(&message) {
                    Ok(v) => v,
                    Err(e) => {
                        console_error!("Failed to deserialize message: {}", e);
                        continue;
                    }
                };

                let invocation_id = match &parsed_message["Value"] {
                    Value::String(s) => s,
                    _ => {
                        console_error!("Failed to get invocation ID from message");
                        continue;
                    }
                };

                let sender: OneshotSender<String>;

                {
                    let mut map = subscribers_clone.borrow_mut();

                    sender = match map.remove(invocation_id) {
                        Some(s) => s,
                        None => {
                            console_log!("No subscriber found for invocation {}", invocation_id);
                            continue;
                        }
                    };
                }

                if let Err(e) = sender.send(message) {
                    console_error!("Failed to send message to subscriber: {}", e);
                    continue;
                }
            }
        });

        Ok(())
    }

    fn handle_invocation_response(message: String, parsed_message: Value) {}
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        let connection = SignalRConnection::new(url);

        Self {
            connection,
            invocation_subscribers: Rc::new(RefCell::new(HashMap::new())),
            invocation_id: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.connection.connect().await.map_err(JsValue::from)?;

        self.start_reader().map_err(JsValue::from)?;

        Ok(())
    }

    pub async fn send_message(&mut self, user: &str, message: &str) -> Result<(), JsValue> {
        self.invocation_id += 1;
        let invocation_id_string = self.invocation_id.to_string();

        self.connection
            .send_invocation(
                invocation_id_string.as_str(),
                "SendMessage",
                &[user, message],
            )
            .map_err(JsValue::from)?;

        self.await_response(invocation_id_string).await?;

        Ok(())
    }

    async fn await_response(&mut self, invocation_id: String) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel::<String>();

        {
            self.invocation_subscribers
                .borrow_mut()
                .insert(invocation_id, sender);
        }

        // TODO: consider timeout
        let message = receiver
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        console_log!("Received invocation reply: {}", message);

        Ok(())
    }
}
