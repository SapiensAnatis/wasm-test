mod utils;

use std::str::RMatches;

use futures::stream::StreamExt;
use signalr_wasm::connection::{Connection, WebSocketEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    console_log!("chat client: WASM loaded.");
}

#[wasm_bindgen]
pub async fn promise() -> Result<(), JsValue> {
    let mut connection = Connection::new("ws://localhost:5095/chatHub");
    connection.connect().await.map_err(|e| JsValue::from(e))?;

    while let Some(event) = connection.event_receiver.next().await {
        match event {
            WebSocketEvent::Message(data) => {
                let string = str::from_utf8(data.as_slice()).unwrap_or("UNABLE TO DECODE UTF8");
                console_log!("Message received: {}", string);
            }
            _ => {
                console_error!("Unhandled event type");
            }
        }
    }

    console_log!("No more events");

    return Ok(());
}
