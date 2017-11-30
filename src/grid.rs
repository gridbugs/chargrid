use std::slice;
use ansi_colour::Colour;
use cgmath::Vector2;
use iterators::*;
use defaults::*;
use view::*;

#[derive(Debug, Clone)]
pub struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Colour,
    bg: Colour,
    bold: bool,
    underline: bool,
}

impl Default for OutputCell {
    fn default() -> Self {
        Self {
            dirty: true,
            ch: DEFAULT_CH,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

impl OutputCell {
    pub fn matches(&self, cell: &Cell) -> bool {
        !self.dirty &&
        self.ch == cell.ch &&
            self.fg == cell.fg &&
            self.bg == cell.bg &&
            self.bold == cell.bold &&
            self.underline == cell.underline
    }
    pub fn copy_fields(&mut self, cell: &Cell) {
        self.dirty = false;
        self.ch = cell.ch;
        self.fg = cell.fg;
        self.bg = cell.bg;
        self.bold = cell.bold;
        self.underline = cell.underline;
    }
}

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
    size: Vector2<u16>,
    cells: Vec<C>,
}

impl<C: Default + Clone> Grid<C> {
    pub fn new(size: Vector2<u16>) -> Self {

        let num_cells = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(num_cells);
        cells.resize(num_cells, Default::default());

        Self {
            size,
            cells,
        }
    }

    pub fn size(&self) -> Vector2<u16> {
        self.size
    }

    pub fn resize(&mut self, size: Vector2<u16>) {
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

    pub fn iter_mut(&mut self) -> slice::IterMut<C> {
        self.cells.iter_mut()
    }
}

impl ViewGrid for Grid<Cell> {
    type Cell = Cell;
    fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut Self::Cell> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }
}
