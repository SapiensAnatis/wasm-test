use wasm_bindgen::prelude::wasm_bindgen;

#[macro_use]
mod log;

mod utils;
mod client;
mod connection;
mod message;

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    console_log!("signalr-wasm: WASM loaded.");
}
