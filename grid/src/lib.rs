#[macro_use] extern crate itertools;
extern crate prototty;
extern crate grid_2d;

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
pub type Iter<'a, C> = grid_2d::Iter<'a, C>;
pub type IterMut<'a, C> = grid_2d::IterMut<'a, C>;
pub type CoordEnumerate<'a, C> = grid_2d::CoordEnumerate<'a, C>;

#[derive(Debug, Clone)]
pub struct Grid<C: Default + Clone> {
    cells: grid_2d::Grid<C>,
    depth: grid_2d::Grid<i32>,
}

impl<C: Default + Clone> Grid<C> {
    pub fn new(size: Size) -> Self {
        let cells = grid_2d::Grid::new_default(size);
        let depth = grid_2d::Grid::new_clone(size, 0);

        Self {
            cells,
            depth,
        }
    }

    pub fn size(&self) -> Size {
        self.cells.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.cells.resize_default(size);
        self.depth.resize_clone(size, 0);
    }

    pub fn clear(&mut self) {
        for (cell, depth) in izip!(self.cells.iter_mut(), self.depth.iter_mut()) {
            *cell = Default::default();
            *depth = 0;
        }
    }

    pub fn enumerate(&self) -> CoordEnumerate<C> {
        self.cells.enumerate()
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
        if let Some(index) = self.depth.coord_to_index(coord) {
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
