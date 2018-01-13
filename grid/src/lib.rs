#[macro_use] extern crate itertools;
extern crate prototty;

use std::slice;
use prototty::*;

pub trait DefaultForeground {
    fn default_foreground() -> Self;
}

pub trait DefaultBackground {
    fn default_background() -> Self;
}

#[derive(Debug, Clone)]
pub struct CommonCell<F, B>
    where F: From<Rgb24> + DefaultForeground,
          B: From<Rgb24> + DefaultBackground,
{
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: F,
    pub background_colour: B,
}

impl<F, B> ViewCell for CommonCell<F, B>
    where F: From<Rgb24> + DefaultForeground,
          B: From<Rgb24> + DefaultBackground,
{
    fn set_character(&mut self, character: char) {
        self.character = character;
    }
    fn set_bold(&mut self, bold: bool) {
        self.bold = bold;
    }
    fn set_underline(&mut self, underline: bool) {
        self.underline = underline;
    }
    fn set_foreground_colour(&mut self, colour: Rgb24) {
        self.foreground_colour = colour.into();
    }
    fn set_background_colour(&mut self, colour: Rgb24) {
        self.background_colour = colour.into();
    }
}

impl<F, B> Default for CommonCell<F, B>
    where F: From<Rgb24> + DefaultForeground,
          B: From<Rgb24> + DefaultBackground,
{
    fn default() -> Self {
        CommonCell {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: F::default_foreground(),
            background_colour: B::default_background(),
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
pub struct Grid<C: Default + Clone> {
    size: Size,
    cells: Vec<C>,
    depth: Vec<i32>,
}

impl<C: Default + Clone> Grid<C> {
    pub fn new(size: Size) -> Self {

        let num_cells = size.count();
        let mut cells = Vec::with_capacity(num_cells);
        cells.resize(num_cells, Default::default());

        let mut depth = Vec::with_capacity(num_cells);
        depth.resize(num_cells, 0);

        Self {
            size,
            cells,
            depth,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn resize(&mut self, size: Size) {
        let num_cells = size.count();
        self.cells.clear();
        self.depth.clear();
        self.cells.resize(num_cells, Default::default());
        self.depth.resize(num_cells, 0);
        self.size = size;
    }

    pub fn clear(&mut self) {
        for (cell, depth) in izip!(self.cells.iter_mut(), self.depth.iter_mut()) {
            *cell = Default::default();
            *depth = 0;
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

impl<C: ViewCell + Default + Clone> ViewGrid for Grid<C> {
    type Cell = C;
    fn get_mut(&mut self, coord: Coord, depth: i32) -> Option<&mut Self::Cell> {
        if let Some(index) = self.size.index(coord) {
            let cell_depth = &mut self.depth[index];
            if depth >= *cell_depth {
                *cell_depth = depth;
                Some(&mut self.cells[index])
            } else {
                None
            }
        } else {
            None
        }
    }
}
