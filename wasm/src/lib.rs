#[macro_use] extern crate itertools;
extern crate prototty;
extern crate prototty_grid;

mod terminal;
mod context;
mod cell;
pub mod input;

pub use context::*;
