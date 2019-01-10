extern crate ansi_colour;
#[macro_use]
extern crate itertools;
extern crate libc;
extern crate term;

extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;

mod cell;
mod context;
mod defaults;
mod error;
mod terminal;

pub use self::context::*;
pub use self::error::*;
pub use prototty_render::{Coord, Size};
