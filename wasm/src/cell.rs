use prototty::{ViewCell, Rgb24, colours};

#[derive(Debug, Clone)]
pub struct Cell {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: u32,
    pub background_colour: u32,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: colours::WHITE.into_u32(),
            background_colour: colours::BLACK.into_u32(),
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
        self.foreground_colour = colour.into_u32();
    }
    fn set_background_colour(&mut self, colour: Rgb24) {
        self.background_colour = colour.into_u32();
    }
}
