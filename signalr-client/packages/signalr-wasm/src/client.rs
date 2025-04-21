use wasm_bindgen::prelude::*;

use crate::connection::SignalRConnection;

type OneshotSender<T> = futures::channel::oneshot::Sender<T>;

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        let connection = SignalRConnection::new(url);

        Self { connection }
    }

    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.connection.connect().await.map_err(JsValue::from)
    }

    pub async fn send_message(&mut self, user: &str, message: &str) -> Result<(), JsValue> {
        self.connection
            .send_invocation(
                "SendMessage".to_owned(),
                vec![user.to_owned(), message.to_owned()],
            )
            .await
            .map_err(JsValue::from)
    }
}
