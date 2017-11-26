use std::slice;
use terminal_colour::Colour;
use cgmath::Vector2;

use iterators::*;
use defaults::*;

#[derive(Debug, Clone)]
pub struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Colour,
    bg: Colour,
}

impl Default for OutputCell {
    fn default() -> Self {
        Self {
            dirty: true,
            ch: DEFAULT_CH,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
        }
    }
}

impl OutputCell {
    pub fn matches(&self, cell: &Cell) -> bool {
        !self.dirty &&
        self.ch == cell.ch &&
            self.fg == cell.fg &&
            self.bg == cell.bg
    }
    pub fn copy_fields(&mut self, cell: &Cell) {
        self.dirty = false;
        self.ch = cell.ch;
        self.fg = cell.fg;
        self.bg = cell.bg;
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    seq: u64,
    pub ch: char,
    depth: i16,
    pub fg: Colour,
    pub bg: Colour,
    bold: bool,
    underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            seq: 0,
            ch: DEFAULT_CH,
            depth: 0,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

impl Cell {
    pub fn update(&mut self, seq: u64, ch: char, depth: i16) {
        if seq > self.seq || (seq == self.seq && depth >= self.depth) {
            self.seq = seq;
            self.ch = ch;
            self.depth = depth;
        }
    }
    pub fn update_with_style(&mut self, seq: u64, ch: char, depth: i16, fg: Colour, bg: Colour, bold: bool, underline: bool) {
        if seq > self.seq || (seq == self.seq && depth >= self.depth) {
            self.seq = seq;
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

    pub fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut C> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }

    pub fn enumerate(&self) -> CoordEnumerate<C> {
        CoordEnumerate::new(CoordIter::new(self.size), self.cells.iter())
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<C> {
        self.cells.iter_mut()
    }
}
