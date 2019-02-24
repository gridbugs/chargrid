use ansi_colour;
use prototty_render::Rgb24;

pub const DEFAULT_CH: char = ' ';
pub const DEFAULT_FG_ANSI_CODE: u8 = ansi_colour::encode_rgb24(Rgb24::new(255, 255, 255));
pub const DEFAULT_BG_ANSI_CODE: u8 = ansi_colour::encode_rgb24(Rgb24::new(0, 0, 0));
