#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate itertools;
extern crate prototty;
extern crate prototty_grid;

mod background;
mod cell;
mod context;
mod formats;
mod input;

pub use context::*;
