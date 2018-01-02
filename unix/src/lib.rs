extern crate libc;
extern crate term;
extern crate ansi_colour;
#[macro_use] extern crate itertools;

extern crate prototty;

mod terminal;
mod error;
mod context;

pub use self::context::*;
pub use self::error::*;
