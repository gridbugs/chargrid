extern crate bincode;
#[macro_use]
extern crate itertools;
extern crate prototty;
extern crate prototty_grid;
extern crate serde;

pub mod alloc;
mod cell;
mod context;
pub mod input;
mod storage;
mod terminal;

pub use context::*;
pub use storage::*;
