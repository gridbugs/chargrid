#[macro_use] extern crate itertools;
extern crate prototty;

use std::slice;
use prototty::*;

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
