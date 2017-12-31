extern crate ansi_colour;
extern crate prototty_coord;

use prototty_coord::{Coord, Size};

pub type Colour = ansi_colour::Colour;

/**
 * A buffered terminal output cell.
 */
pub trait ViewCell {
    fn update(&mut self, ch: char, depth: i32);
    fn update_with_colour(&mut self, ch: char, depth: i32, fg: Colour, bg: Colour);
    fn update_with_style(&mut self, ch: char, depth: i32, fg: Colour, bg: Colour,
                         bold: bool, underline: bool);
}

/**
 * A grid of cells which implement `ViewCell`.
 */
pub trait ViewGrid {
    type Cell: ViewCell;
    fn get_mut(&mut self, coord: Coord) -> Option<&mut Self::Cell>;
}

/**
 * Defines a method for rendering a `T` to the terminal.
 */
pub trait View<T> {
    /**
     * Update the cells in `grid` to describe how a type should be rendered.
     */
    fn view<G: ViewGrid>(&self, data: &T, offset: Coord, depth: i32, grid: &mut G);
}

/**
 * Report the size of a `T` when rendered.
 */
pub trait ViewSize<T> {
    /**
     * Returns the size in cells of the rectangle containing a ui element.
     * This allows for the implementation of decorator ui components that
     * render a border around some inner element.
     */
    fn size(&self, data: &T) -> Size;
}

pub trait Renderer {
    type Error: ::std::fmt::Debug;
    fn render<V: View<T>, T>(&mut self, view: &V, data: &T) -> Result<(), Self::Error>;
}
