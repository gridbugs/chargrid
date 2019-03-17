use defaults::*;
use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub foreground: Rgb24,
    pub background: Rgb24,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            foreground: DEFAULT_FG,
            background: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

impl Style {
    pub fn view_cell(&self, character: char) -> ViewCell {
        ViewCell {
            character: Some(character),
            foreground: Some(self.foreground),
            background: Some(self.background),
            bold: Some(self.bold),
            underline: Some(self.underline),
        }
    }
}
