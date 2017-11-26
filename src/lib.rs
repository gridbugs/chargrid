extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate libc;
extern crate term;
extern crate cgmath;
extern crate ansi_colour;
#[macro_use] extern crate itertools;

mod core;
mod iterators;
mod defaults;
mod grid;
mod error;
mod input;
mod elements;
mod context;

pub use self::error::{Result, Error};
pub use self::core::terminal::Terminal;
pub use self::input::Input;
pub use self::context::*;
pub use self::iterators::*;
pub use self::elements::*;
