use futures::StreamExt;
use signalr_wasm::connection::SignalRConnection;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
}

#[wasm_bindgen]
impl ChatClient {
    pub async fn infinite_read(self: &mut Self) {
        while let Some(event) = self.connection.event_receiver.next().await {
            console_log!("{:?}", event);
        }
    }
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        let connection = SignalRConnection::new(url);

        return Self { connection };
    }

    pub async fn connect(self: &mut Self) -> Result<(), JsValue> {
        return self.connection.connect().await.map_err(|e| e.into());
    }
}
