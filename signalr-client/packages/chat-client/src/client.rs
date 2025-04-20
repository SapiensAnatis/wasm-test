use std::rc::Rc;

use async_broadcast::Receiver;
use signalr_wasm::connection::{SignalRConnection, WebSocketEvent};
use std::cell::RefCell;
use std::ops::Deref;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, spawn_local};

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
    message_subscribers: Rc<RefCell<Vec<js_sys::Function>>>,
}

impl ChatClient {
    pub fn start_read(
        mut receiver: Receiver<WebSocketEvent>,
        subscribers: Rc<RefCell<Vec<js_sys::Function>>>,
    ) {
        console_log!("Starting read loop");

        spawn_local(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    WebSocketEvent::Message(data) => {
                        let data_str =
                            str::from_utf8(data.as_slice()).unwrap_or("UNABLE TO DECODE");
                        console_log!("Received message: {}", data_str);

                        let subscribers_vec = subscribers.borrow();
                        let this = JsValue::null();

                        for subscriber in subscribers_vec.deref() {
                            match subscriber.call1(&this, &JsValue::from(data_str)) {
                                Ok(_) => {}
                                Err(e) => console_error!("Failed to call subscriber: {:?}", e),
                            }
                        }
                    }
                    _ => {
                        console_log!("Received event: {:?}", event);
                    }
                }
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
        };
    }

    pub async fn connect(self: &mut Self) -> Result<(), JsValue> {
        self.connection
            .connect()
            .await
            .map_err(|e| JsValue::from(e))?;

        return Ok(());
    }

    pub fn register_callback(self: &mut Self, on_message: js_sys::Function) {
        let mut sub_ref = self.message_subscribers.borrow_mut();
        sub_ref.push(on_message);
    }
}
