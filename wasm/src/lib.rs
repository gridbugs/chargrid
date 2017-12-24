#[macro_use] extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[macro_use] extern crate itertools;
        extern crate prototty_traits;
        extern crate prototty_grid;

        mod terminal;
        mod context;

        pub use self::context::*;
    }
}
