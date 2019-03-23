extern crate prototty_render;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod rich_text;
mod text;
pub mod wrap;
pub use rich_text::*;
pub use text::*;
