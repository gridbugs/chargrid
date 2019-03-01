#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
#[cfg(feature = "storage")]
extern crate prototty_file_storage;
extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;

mod background;
mod context;
mod formats;
mod input;

pub use context::*;
pub use prototty_render::{Coord, Size};

#[cfg(feature = "storage")]
pub use prototty_file_storage::{FileStorage, LoadError, Storage, StoreError};
