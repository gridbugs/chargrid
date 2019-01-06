use prototty_render::{Rgb24, ViewCellInfo};

/// Rich text settings
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct TextInfo {
    pub foreground_colour: Option<Rgb24>,
    pub background_colour: Option<Rgb24>,
    pub underline: bool,
    pub bold: bool,
}

impl Default for TextInfo {
    fn default() -> Self {
        Self {
            foreground_colour: None,
            background_colour: None,
            underline: false,
            bold: false,
        }
    }
}

impl TextInfo {
    pub fn foreground_colour(self, colour: Rgb24) -> Self {
        Self {
            foreground_colour: Some(colour),
            ..self
        }
    }
    pub fn background_colour(self, colour: Rgb24) -> Self {
        Self {
            background_colour: Some(colour),
            ..self
        }
    }
    pub fn underline(self) -> Self {
        Self {
            underline: true,
            ..self
        }
    }
    pub fn bold(self) -> Self {
        Self { bold: true, ..self }
    }
    pub fn view_cell_info(&self, character: char) -> ViewCellInfo {
        ViewCellInfo {
            character: Some(character),
            foreground: self.foreground_colour,
            background: self.background_colour,
            underline: Some(self.underline),
            bold: Some(self.bold),
        }
    }
}
