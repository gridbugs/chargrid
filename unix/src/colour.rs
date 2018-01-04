use prototty::Rgb24;
use ansi_colour::*;

pub fn rgb24_to_ansi(rgb24: Rgb24) -> Colour {
    let red = rgb24.red as u32 * MAX_RGB_CHANNEL as u32 / 255;
    let green = rgb24.green as u32 * MAX_RGB_CHANNEL as u32 / 255;
    let blue = rgb24.blue as u32 * MAX_RGB_CHANNEL as u32 / 255;

    Colour::rgb(red as u8, green as u8, blue as u8).unwrap()
}
