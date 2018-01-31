use coord::{Coord, Size};
use rgb24::Rgb24;

/// A cell that a view can write to
pub trait ViewCell {
    fn set_character(&mut self, character: char);
    fn set_bold(&mut self, bold: bool);
    fn set_underline(&mut self, underline: bool);
    fn set_foreground_colour(&mut self, colour: Rgb24);
    fn set_background_colour(&mut self, colour: Rgb24);
}

/// A grid of cells
pub trait ViewGrid {
    type Cell: ViewCell;
    fn get_mut(&mut self, coord: Coord, depth: i32) -> Option<&mut Self::Cell>;
}

/// Defines a method for rendering a `T` to the terminal.
pub trait View<T> {
     /// Update the cells in `grid` to describe how a type should be rendered.
    fn view<G: ViewGrid>(&mut self, data: &T, offset: Coord, depth: i32, grid: &mut G);
}

/// Report the size of a `T` when rendered.
pub trait ViewSize<T> {
     /// Returns the size in cells of the rectangle containing a ui element.
     /// This allows for the implementation of decorator ui components that
     /// render a border around some inner element.
    fn size(&mut self, data: &T) -> Size;
}
