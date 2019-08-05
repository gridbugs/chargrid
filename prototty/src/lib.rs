pub extern crate prototty_decorator as decorator;
pub extern crate prototty_event_routine as event_routine;
pub extern crate prototty_input as input;
pub extern crate prototty_menu as menu;
pub extern crate prototty_render as render;
#[cfg(feature = "storage")]
pub extern crate prototty_storage as storage;
pub extern crate prototty_text as text;
pub use render::{Coord, Size};

#[cfg(feature = "wasm_log")]
extern crate wasm_bindgen;
#[cfg(feature = "wasm_log")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm_log")]
#[cfg_attr(feature = "wasm_log", wasm_bindgen)]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[cfg(feature = "wasm_log")]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(not(feature = "wasm_log"))]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (eprintln!($($t)*))
}
