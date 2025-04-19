use async_broadcast::Receiver;
use signalr_wasm::connection::{SignalRConnection, WebSocketEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub struct ChatClient {
    connection: SignalRConnection,
}

impl ChatClient {
    pub fn start_read(mut receiver: Receiver<WebSocketEvent>) {
        console_log!("Starting read loop");

        spawn_local(async move {
            loop {
                while let Ok(event) = receiver.recv().await {
                    match event {
                        WebSocketEvent::Message(data) => {
                            console_log!(
                                "Received message: {}",
                                str::from_utf8(data.as_slice()).unwrap_or("UNABLE TO DECODE")
                            )
                        }
                        _ => {
                            console_log!("Received event: {:?}", event);
                        }
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

        return Self { connection };
    }

    pub async fn connect(self: &mut Self) -> Result<(), JsValue> {
        self.connection
            .connect()
            .await
            .map_err(|e| JsValue::from(e))?;
        Self::start_read(self.connection.event_receiver.clone());

        return Ok(());
    }
}
