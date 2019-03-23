extern crate coord_2d;
extern crate rgb24;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod context;
mod view;

pub use context::*;
pub use coord_2d::{Coord, Size};
pub use rgb24::*;
pub use view::*;

pub mod colours {
    use rgb24::*;

    pub const BLACK: Rgb24 = rgb24(0, 0, 0);
    pub const RED: Rgb24 = rgb24(187, 0, 0);
    pub const GREEN: Rgb24 = rgb24(0, 187, 0);
    pub const YELLOW: Rgb24 = rgb24(187, 187, 0);
    pub const BLUE: Rgb24 = rgb24(0, 0, 187);
    pub const MAGENTA: Rgb24 = rgb24(187, 0, 187);
    pub const CYAN: Rgb24 = rgb24(0, 187, 187);
    pub const GREY: Rgb24 = rgb24(187, 187, 187);
    pub const DARK_GREY: Rgb24 = rgb24(85, 85, 85);
    pub const BRIGHT_RED: Rgb24 = rgb24(255, 85, 85);
    pub const BRIGHT_GREEN: Rgb24 = rgb24(0, 255, 0);
    pub const BRIGHT_YELLOW: Rgb24 = rgb24(255, 255, 85);
    pub const BRIGHT_BLUE: Rgb24 = rgb24(85, 85, 255);
    pub const BRIGHT_MAGENTA: Rgb24 = rgb24(255, 85, 255);
    pub const BRIGHT_CYAN: Rgb24 = rgb24(85, 255, 255);
    pub const WHITE: Rgb24 = rgb24(255, 255, 255);
}
