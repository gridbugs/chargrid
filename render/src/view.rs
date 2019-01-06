use super::{Coord, Size};
use rgb24::Rgb24;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewCellInfo {
    pub character: Option<char>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub foreground: Option<Rgb24>,
    pub background: Option<Rgb24>,
}

impl ViewCellInfo {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_character(mut self, character: char) -> Self {
        self.character = Some(character);
        self
    }
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = Some(underline);
        self
    }
    pub fn with_foreground(mut self, foreground: Rgb24) -> Self {
        self.foreground = Some(foreground);
        self
    }
    pub fn with_background(mut self, background: Rgb24) -> Self {
        self.background = Some(background);
        self
    }
}

/// A grid of cells
pub trait ViewGrid {
    fn set_cell(&mut self, coord: Coord, depth: i32, info: ViewCellInfo);
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
