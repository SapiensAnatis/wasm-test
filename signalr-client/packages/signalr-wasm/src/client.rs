use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::connection::SignalRConnection;

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
    user: String,
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str, user: String) -> Self {
        let connection = SignalRConnection::new(url);

        Self { connection, user }
    }

    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.connection.connect().await.map_err(JsValue::from)
    }

    pub fn on_message_received(&mut self, callback: Function) -> () {
        self.connection
            .on("ReceiveMessage", move |user: String, message: String| {
                let this = JsValue::null();
                let user_val = JsValue::from(user);
                let message_val = JsValue::from(message);

                if let Err(e) = callback.call2(&this, &user_val, &message_val) {
                    console_error!("Failed to invoke on_message_received: {:?}", e);
                }
            });
    }

    pub fn set_user(&mut self, user: String) {
        self.user = user;
    }

    pub async fn send_message(&mut self, message: &str) -> Result<(), JsValue> {
        self.connection
            .send_invocation(
                "SendMessage".to_owned(),
                vec![
                    Value::String(self.user.clone()),
                    Value::String(message.to_owned()),
                ],
            )
            .await
            .map_err(JsValue::from)
    }
}
