use defaults::*;
use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub foreground: Option<Rgb24>,
    pub background: Option<Rgb24>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            foreground: Some(DEFAULT_FG),
            background: Some(DEFAULT_BG),
            bold: None,
            underline: None,
        }
    }
}

impl Style {
    pub fn view_cell(&self, character: char) -> ViewCell {
        ViewCell {
            character: Some(character),
            foreground: self.foreground,
            background: self.background,
            bold: self.bold,
            underline: self.underline,
        }
    }
}
