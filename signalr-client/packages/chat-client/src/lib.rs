#[macro_use]
mod log;

mod client;
mod message;
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    console_log!("chat client: WASM loaded.");
}
