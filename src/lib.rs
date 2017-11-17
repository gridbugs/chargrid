extern crate libc;
extern crate term;
extern crate cgmath;
extern crate terminal_colour;

mod unix_backend;
mod terminal;
mod error;
mod term_info_cache;

pub use self::error::{Result, Error};
pub use self::terminal::Terminal;
