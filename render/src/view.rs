use super::{Coord, Size};
use rgb24::Rgb24;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewCell {
    pub character: Option<char>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub foreground: Option<Rgb24>,
    pub background: Option<Rgb24>,
}

impl ViewCell {
    pub const fn new() -> Self {
        Self {
            character: None,
            bold: None,
            underline: None,
            foreground: None,
            background: None,
        }
    }
    pub const fn with_character(self, character: char) -> Self {
        Self {
            character: Some(character),
            ..self
        }
    }
    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            bold: Some(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            underline: Some(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgb24) -> Self {
        Self {
            foreground: Some(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgb24) -> Self {
        Self {
            background: Some(background),
            ..self
        }
    }
    pub fn set_character(&mut self, character: char) {
        self.character = Some(character);
    }
    pub fn clear_character(&mut self) {
        self.character = None;
    }
    pub fn set_bold(&mut self, bold: bool) {
        self.bold = Some(bold);
    }
    pub fn clear_bold(&mut self) {
        self.bold = None;
    }
    pub fn set_underline(&mut self, underline: bool) {
        self.underline = Some(underline);
    }
    pub fn clear_underline(&mut self) {
        self.underline = None;
    }
    pub fn set_foreground(&mut self, foreground: Rgb24) {
        self.foreground = Some(foreground);
    }
    pub fn clear_foreground(&mut self) {
        self.foreground = None;
    }
    pub fn set_background(&mut self, background: Rgb24) {
        self.background = Some(background);
    }
    pub fn clear_background(&mut self) {
        self.background = None;
    }
    pub fn coalesce(&mut self, other: &Self) {
        if self.character.is_none() {
            self.character = other.character;
        }
        if self.bold.is_none() {
            self.bold = other.bold;
        }
        if self.underline.is_none() {
            self.underline = other.bold;
        }
        if self.foreground.is_none() {
            self.foreground = other.foreground;
        }
        if self.background.is_none() {
            self.background = other.background;
        }
    }
}

/// A grid of cells
pub trait ViewGrid {
    fn set_cell(&mut self, coord: Coord, depth: i32, info: ViewCell);
    fn size(&self) -> Size;
}

/// Defines a method for rendering a `T` to the terminal.
pub trait View<T: ?Sized> {
    /// Update the cells in `grid` to describe how a type should be rendered.
    /// This mutably borrows `self` to allow the view to contain buffers/caches which
    /// are updated during rendering.
    fn view<G: ViewGrid>(&mut self, data: &T, offset: Coord, depth: i32, grid: &mut G);
}

/// Report the size of a `T` when rendered.
pub trait ViewSize<T: ?Sized> {
    /// Returns the size in cells of the rectangle containing a ui element.
    /// This allows for the implementation of decorator ui components that
    /// render a border around some inner element.
    fn size(&mut self, data: &T) -> Size;
}
