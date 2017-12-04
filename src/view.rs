use cgmath::Vector2;
use ansi_colour::Colour;

/**
 * A buffered terminal output cell.
 */
pub trait ViewCell {
    fn update(&mut self, ch: char, depth: i16);
    fn update_with_colour(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour);
    fn update_with_style(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour,
                         bold: bool, underline: bool);
}

/**
 * A grid of cells which implement `ViewCell`.
 */
pub trait ViewGrid {
    type Cell: ViewCell;
    fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut Self::Cell>;
}

/**
 * Defines how to render a type to the terminal.
 */
pub trait View {
    /**
     * Update the cells in `grid` to describe how a type should be rendered.
     * Implementations of `view` for low level ui components will typically
     * involve updating cells directly. Implementations for higer level
     * components, such as an entire application's ui, will typically call
     * the `view` methed of lower level components which make up the ui.
     */
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G);
}

pub trait ViewSize {
    /**
     * Returns the size in cells of the rectangle containing a ui element.
     * This allows for the implementation of decorator ui components that
     * render a border around some inner element.
     */
    fn size(&self) -> Vector2<u16>;
}
