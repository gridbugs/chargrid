use prototty::{ViewCell, Rgb24};
use ansi_colour::*;
use defaults::*;
use colour::*;

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
            character: DEFAULT_CH,
            bold: false,
            underline: false,
            foreground_colour: Colour::from_code(DEFAULT_FG_ANSI_CODE),
            background_colour: Colour::from_code(DEFAULT_BG_ANSI_CODE),
        }
    }
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
