extern crate libc;
extern crate term;

pub extern crate prototty_event_routine;
extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;

mod context;
mod error;
mod terminal;

pub use self::context::*;
pub use self::error::*;
pub use self::terminal::col_encode;
pub use self::terminal::ColEncode;
pub use prototty_render::{grey24, rgb24, Coord, Rgb24, Size};
