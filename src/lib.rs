extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate libc;
extern crate term;
extern crate cgmath;
extern crate terminal_colour;

mod unix_backend;
mod terminal;
mod error;
mod term_info_cache;
mod byte_prefix_tree;
mod input;

pub use self::error::{Result, Error};
pub use self::terminal::Terminal;
pub use self::input::Input;
