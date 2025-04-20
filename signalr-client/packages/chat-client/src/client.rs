use std::rc::Rc;

use futures::StreamExt;
use signalr_wasm::connection::SignalRConnection;
use std::cell::RefCell;
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

        Self {
            connection,
            message_subscribers: Rc::new(RefCell::new(vec![])),
            current_invocation: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.connection.connect().await.map_err(JsValue::from)?;

        self.start_read();

        Ok(())
    }

    pub fn register_callback(&mut self, on_message: js_sys::Function) {
        let mut sub_ref = self.message_subscribers.borrow_mut();
        sub_ref.push(on_message);
    }

    pub fn send_message(&mut self, user: String, message: String) -> Result<(), String> {
        self.connection
            .send_invocation("SendMessage".to_owned(), vec![user, message])
    }
}
