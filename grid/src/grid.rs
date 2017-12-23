use std::slice;
use iterators::*;
use prototty_defaults::*;
use prototty_traits::*;

pub type Iter<'a, C> = slice::Iter<'a, C>;
pub type IterMut<'a, C> = slice::IterMut<'a, C>;

#[derive(Debug, Clone)]
pub struct Cell {
    pub ch: char,
    depth: i16,
    pub fg: Colour,
    pub bg: Colour,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: DEFAULT_CH,
            depth: 0,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

impl ViewCell for Cell {
    fn update(&mut self, ch: char, depth: i16) {
        if depth >= self.depth {
            self.ch = ch;
            self.depth = depth;
        }
    }
    fn update_with_colour(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour) {
        if depth >= self.depth {
            self.ch = ch;
            self.depth = depth;
            self.fg = fg;
            self.bg = bg;
        }
    }
    fn update_with_style(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour, bold: bool, underline: bool) {
        if depth >= self.depth {
            self.ch = ch;
            self.depth = depth;
            self.fg = fg;
            self.bg = bg;
            self.bold = bold;
            self.underline = underline;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid<C> {
    size: Size,
    cells: Vec<C>,
}

impl<C: Default + Clone> Grid<C> {
    pub fn new(size: Size) -> Self {

        let num_cells = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(num_cells);
        cells.resize(num_cells, Default::default());

        Self {
            size,
            cells,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn resize(&mut self, size: Size) {
        let num_cells = (size.x * size.y) as usize;
        self.cells.resize(num_cells, Default::default());
        self.size = size;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Default::default();
        }
    }

    pub fn enumerate(&self) -> CoordEnumerate<C> {
        CoordEnumerate::new(CoordIter::new(self.size), self.cells.iter())
    }

    pub fn iter(&self) -> Iter<C> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<C> {
        self.cells.iter_mut()
    }
}

impl ViewGrid for Grid<Cell> {
    type Cell = Cell;
    fn get_mut(&mut self, coord: Coord) -> Option<&mut Self::Cell> {
        let coord: Size = coord.cast().unwrap();

        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }
}
