use prototty::{ViewCell, Rgb24};

/// Rich text settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
        Self { foreground_colour: Some(colour), .. self }
    }
    pub fn background_colour(self, colour: Rgb24) -> Self {
        Self { background_colour: Some(colour), .. self }
    }
    pub fn underline(self) -> Self {
        Self { underline: true, .. self }
    }
    pub fn bold(self) -> Self {
        Self { bold: true, .. self }
    }
    pub fn write_cell<C: ViewCell>(&self, cell: &mut C) {
        if let Some(foreground_colour) = self.foreground_colour {
            cell.set_foreground_colour(foreground_colour);
        }
        if let Some(background_colour) = self.background_colour {
            cell.set_background_colour(background_colour);
        }
        cell.set_bold(self.bold);
        cell.set_underline(self.underline);
    }
}
