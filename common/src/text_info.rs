use defaults::*;
use prototty::{ViewCell, Rgb24};

/// Rich text settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextInfo {
    pub foreground_colour: Rgb24,
    pub background_colour: Rgb24,
    pub underline: bool,
    pub bold: bool,
}

impl Default for TextInfo {
    fn default() -> Self {
        Self {
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            underline: false,
            bold: false,
        }
    }
}

impl TextInfo {
    pub fn foreground_colour(self, colour: Rgb24) -> Self {
        Self { foreground_colour: colour, .. self }
    }
    pub fn backrgound_colour(self, colour: Rgb24) -> Self {
        Self { background_colour: colour, .. self }
    }
    pub fn underline(self) -> Self {
        Self { underline: true, .. self }
    }
    pub fn bold(self) -> Self {
        Self { bold: true, .. self }
    }
    pub fn write_cell<C: ViewCell>(&self, cell: &mut C) {
        cell.set_foreground_colour(self.foreground_colour);
        cell.set_background_colour(self.background_colour);
        cell.set_bold(self.bold);
        cell.set_underline(self.underline);
    }
}
