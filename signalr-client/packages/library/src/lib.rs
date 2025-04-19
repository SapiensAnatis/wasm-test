#[macro_use]
mod log;

mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello to the console");
    alert("Hello from Rust!");
}
