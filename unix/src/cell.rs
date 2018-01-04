use prototty::{ViewCell, Rgb24};
use ansi_colour::*;

#[derive(Debug, Clone)]
pub struct Cell {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: Colour,
    pub background_colour: Colour,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: colours::WHITE,
            background_colour: colours::BLACK,
        }
    }
}

fn rgb24_to_ansi(rgb24: Rgb24) -> Colour {
    let red = rgb24.red as u32 * NUM_RGB_COLOURS_PER_CHANNEL as u32 / 255;
    let green = rgb24.green as u32 * NUM_RGB_COLOURS_PER_CHANNEL as u32 / 255;
    let blue = rgb24.blue as u32 * NUM_RGB_COLOURS_PER_CHANNEL as u32 / 255;

    Colour::rgb(red as u8, green as u8, blue as u8).unwrap()
}

impl ViewCell for Cell {
    fn set_character(&mut self, character: char) {
        self.character = character;
    }
    fn set_bold(&mut self, bold: bool) {
        self.bold = bold;
    }
    fn set_underline(&mut self, underline: bool) {
        self.underline = underline;
    }
    fn set_foreground_colour(&mut self, colour: Rgb24) {
        self.foreground_colour = rgb24_to_ansi(colour);
    }
    fn set_background_colour(&mut self, colour: Rgb24) {
        self.background_colour = rgb24_to_ansi(colour);
    }
}
