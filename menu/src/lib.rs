extern crate prototty_input;
extern crate prototty_render;
extern crate prototty_text;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod menu;
pub use menu::*;
