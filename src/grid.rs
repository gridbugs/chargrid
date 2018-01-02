use std::slice;
use ansi_colour::Colour;
use default::*;
use view::*;
use coord::*;

#[derive(Debug, Clone)]
pub struct Cell {
    depth: i32,
    pub character: char,
    pub foreground_colour: Colour,
    pub background_colour: Colour,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            depth: 0,
            character: DEFAULT_CH,
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}
pub type Iter<'a, C> = slice::Iter<'a, C>;
pub type IterMut<'a, C> = slice::IterMut<'a, C>;

pub struct CoordEnumerate<'a, T: 'a> {
    coords: CoordIter,
    iter: slice::Iter<'a, T>,
}

impl<'a, T> CoordEnumerate<'a, T> {
    fn new(coords: CoordIter, iter: slice::Iter<'a, T>) -> Self {
        Self { coords, iter }
    }
}

impl<'a, T> Iterator for CoordEnumerate<'a, T> {
    type Item = (Coord, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.coords.next().and_then(|c| {
            self.iter.next().map(|t| (c, t))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Grid<C> {
    size: Size,
    cells: Vec<C>,
}

impl<C: Default + Clone> Grid<C> {
    pub fn new(size: Size) -> Self {

        let num_cells = size.count();
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
        let num_cells = size.count();
        self.cells.resize(num_cells, Default::default());
        self.size = size;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Default::default();
        }
    }

    pub fn enumerate(&self) -> CoordEnumerate<C> {
        CoordEnumerate::new(self.size.coords(), self.cells.iter())
    }

    pub fn iter(&self) -> Iter<C> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<C> {
        self.cells.iter_mut()
    }
}

impl ViewGrid for Grid<Cell> {
    fn get_mut(&mut self, coord: Coord, depth: i32) -> Option<&mut Cell> {
        if let Some(index) = self.size.index(coord) {
            let cell = &mut self.cells[index];
            if depth >= cell.depth {
                cell.depth = depth;
                Some(cell)
            } else {
                None
            }
        } else {
            None
        }
    }
}
