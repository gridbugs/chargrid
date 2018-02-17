extern crate bincode;
#[macro_use]
extern crate itertools;
extern crate prototty;
extern crate prototty_grid;
extern crate serde;

mod terminal;
mod context;
mod cell;
mod storage;
pub mod input;
pub mod alloc;

pub use context::*;
pub use storage::*;
