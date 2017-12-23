pub extern crate prototty_traits as traits;
pub extern crate prototty_menu as menu;
pub extern crate prototty_decorator as decorator;
pub extern crate prototty_text as text;
pub extern crate prototty_input as input;

#[cfg(unix)]
pub extern crate prototty_unix as unix;

#[cfg(target_arch = "wasm32")]
pub extern crate prototty_wasm as wasm;
