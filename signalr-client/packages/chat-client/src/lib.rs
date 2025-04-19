mod utils;

use signalr_wasm::connection::Connection;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn start() {
    let mut connection = Connection::new("ws://localhost:5095/chatHub");
    let connect_result = connection.connect();

    match connect_result {
        Ok(()) => console_log!("Created websocket"),
        Err(e) => console_error!("Failed to create websocket: {}", e),
    }
}
