extern crate libc;
extern crate term;
extern crate cgmath;

mod unix_backend;
mod terminal;
mod error;
mod term_info_strings;

pub use self::error::{Result, Error};
pub use self::terminal::Terminal;
