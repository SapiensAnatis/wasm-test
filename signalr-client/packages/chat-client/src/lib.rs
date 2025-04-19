mod utils;

use signalr_wasm::connection::Connection;
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
    let connect_future = connection.connect().map_err(|e| JsValue::from(e))?;

    connect_future.await.map_err(|e| JsValue::from(e))
}
