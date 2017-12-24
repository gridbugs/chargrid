#[macro_use] extern crate cfg_if;

cfg_if! {
    if #[cfg(unix)] {
        extern crate libc;
        extern crate term;
        extern crate cgmath;
        extern crate ansi_colour;
        #[macro_use] extern crate itertools;

        extern crate prototty_defaults;
        extern crate prototty_input;
        extern crate prototty_traits;
        extern crate prototty_grid;

        mod terminal;
        mod error;
        mod context;

        pub use self::context::*;
        pub use self::error::*;
    }
}
