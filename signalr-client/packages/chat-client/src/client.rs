use std::{ops::Index, rc::Rc};

use futures::StreamExt;
use signalr_wasm::connection::{SignalRConnection, WebSocketEvent};
use std::cell::RefCell;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
    message_subscribers: Rc<RefCell<Vec<js_sys::Function>>>,
    current_invocation: i32,
}

impl ChatClient {
    pub fn start_read(&mut self) {
        console_log!("Starting read loop");

        let mut receiver = match self.connection.open_message_channel() {
            Ok(recv) => recv,
            Err(e) => {
                console_error!("Failed to open message channel: {}", e);
                return;
            }
        };

        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                console_log!("Received message: {}", message);
            }
        });
    }
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        let connection = SignalRConnection::new(url);

        return Self {
            connection,
            message_subscribers: Rc::new(RefCell::new(vec![])),
            current_invocation: 0,
        };
    }

    pub async fn connect(self: &mut Self) -> Result<(), JsValue> {
        self.connection
            .connect()
            .await
            .map_err(|e| JsValue::from(e))?;

        self.start_read();

        return Ok(());
    }

    pub fn register_callback(self: &mut Self, on_message: js_sys::Function) {
        let mut sub_ref = self.message_subscribers.borrow_mut();
        sub_ref.push(on_message);
    }
}
