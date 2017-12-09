use std::slice;
use prototty::*;
use cgmath::Vector2;
use ansi_colour::Colour;
use defaults::*;
use super::iterators::*;

const BLANK: char = ' ';

/// Iterator over cells in a `Canvas`.
pub type Iter<'a> = slice::Iter<'a, CanvasCell>;

/// Iterator over cells in a `Canvas`
/// allowing modification.
pub type IterMut<'a> = slice::IterMut<'a, CanvasCell>;

/// A single cell of a `Canvas`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CanvasCell {
    pub character: char,
    pub foreground_colour: Colour,
    pub background_colour: Colour,
    pub bold: bool,
    pub underline: bool,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            character: BLANK,
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

/// A rectangular drawing element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    size: Vector2<u16>,
    cells: Vec<CanvasCell>,
}

impl Canvas {
    pub fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        let size = size.into();
        let capacity = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Default::default());
        Self {
            size,
            cells,
        }
    }

    /// Returns a mutable reference to a cell at the given coordinate.
    pub fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut CanvasCell> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }

    /// Returns a reference to a cell at the given coordinate.
    pub fn get(&self, coord: Vector2<i16>) -> Option<&CanvasCell> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&self.cells[(coord.y * self.size.x + coord.x) as usize])
    }

    /// Returns an iterator over the coordinates of the canvas.
    pub fn coords(&self) -> CoordIter {
        CoordIter::new(self.size)
    }

    /// Returns an iterator over the cells in the canvas.
    pub fn iter(&self) -> Iter {
        self.cells.iter()
    }

    /// Returns a iterator that allows modifying each cell.
    pub fn iter_mut(&mut self) -> IterMut {
        self.cells.iter_mut()
    }

    /// Returns an iterator over pairs of coordinates and cells.
    pub fn enumerate(&self) -> CoordEnumerate<CanvasCell> {
        CoordEnumerate::new(self.coords(), self.iter())
    }

    /// Returns an iterator over pairs of coordinates and cells
    /// allowing modification of the cells.
    pub fn enumerate_mut(&mut self) -> CoordEnumerateMut<CanvasCell> {
        CoordEnumerateMut::new(self.coords(), self.iter_mut())
    }
}

impl View for Canvas {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        for (coord, canvas_cell) in izip!(CoordIter::new(self.size), self.cells.iter()) {
            let coord = offset + coord;
            if let Some(terminal_cell) = grid.get_mut(coord) {
                terminal_cell.update_with_style(canvas_cell.character, depth,
                                                canvas_cell.foreground_colour,
                                                canvas_cell.background_colour,
                                                canvas_cell.bold,
                                                canvas_cell.underline);
            }
        }
    }
}

impl ViewSize for Canvas {
    fn size(&self) -> Vector2<u16> { self.size }
}
