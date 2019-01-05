#[macro_use]
extern crate itertools;
extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;

pub mod alloc;
mod cell;
mod context;
pub mod input;
mod terminal;

pub use context::*;
