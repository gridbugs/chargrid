extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate libc;
extern crate term;
extern crate cgmath;
extern crate ansi_colour;
#[macro_use] extern crate itertools;

mod core;
mod grid;
mod error;
mod input;
mod context;
mod defaults;
mod view;
mod iterators;

pub use self::error::{Result, Error};
pub use self::input::Input;
pub use self::context::*;
pub use self::view::*;
