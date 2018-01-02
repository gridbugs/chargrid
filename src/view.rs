use coord::{Coord, Size};
use grid::Cell;

/// A grid of cells
pub trait ViewGrid {
    fn get_mut(&mut self, coord: Coord, depth: i32) -> Option<&mut Cell>;
}

/// Defines a method for rendering a `T` to the terminal.
pub trait View<T> {
     /// Update the cells in `grid` to describe how a type should be rendered.
    fn view<G: ViewGrid>(&self, data: &T, offset: Coord, depth: i32, grid: &mut G);
}

/// Report the size of a `T` when rendered.
pub trait ViewSize<T> {
     /// Returns the size in cells of the rectangle containing a ui element.
     /// This allows for the implementation of decorator ui components that
     /// render a border around some inner element.
    fn size(&self, data: &T) -> Size;
}

/// Trait to implement for renderers that take data and a view that knows how
/// to render the data to a grid, and actually draws the result.
pub trait Renderer {
    type Error: ::std::fmt::Debug;
    fn render<V: View<T>, T>(&mut self, view: &V, data: &T) -> Result<(), Self::Error>;
}
