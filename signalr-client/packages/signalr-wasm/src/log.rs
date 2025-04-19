use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (crate::log::log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (crate::log::error(&format_args!($($t)*).to_string()))
}

pub(crate) use console_error;
pub(crate) use console_log;
