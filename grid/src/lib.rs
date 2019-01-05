pub extern crate grid_2d;
extern crate prototty_render;

use prototty_render::*;

pub trait DefaultForeground {
    fn default_foreground() -> Self;
}

pub trait DefaultBackground {
    fn default_background() -> Self;
}

#[derive(Debug, Clone)]
pub struct CommonCell<F, B>
where
    F: From<Rgb24> + DefaultForeground,
    B: From<Rgb24> + DefaultBackground,
{
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: F,
    pub background_colour: B,
    foreground_depth: i32,
    background_depth: i32,
    access_depth: i32,
}

impl<F, B> ViewCell for CommonCell<F, B>
where
    F: From<Rgb24> + DefaultForeground,
    B: From<Rgb24> + DefaultBackground,
{
    fn set_character(&mut self, character: char) {
        if self.access_depth >= self.foreground_depth {
            self.character = character;
            self.foreground_depth = self.access_depth;
        }
    }
    fn set_bold(&mut self, bold: bool) {
        if self.access_depth >= self.foreground_depth {
            self.bold = bold;
            self.foreground_depth = self.access_depth;
        }
    }
    fn set_underline(&mut self, underline: bool) {
        if self.access_depth >= self.foreground_depth {
            self.underline = underline;
            self.foreground_depth = self.access_depth;
        }
    }
    fn set_foreground_colour(&mut self, colour: Rgb24) {
        if self.access_depth >= self.foreground_depth {
            self.foreground_colour = colour.into();
            self.foreground_depth = self.access_depth;
        }
    }
    fn set_background_colour(&mut self, colour: Rgb24) {
        if self.access_depth >= self.background_depth {
            self.background_colour = colour.into();
            self.background_depth = self.access_depth;
        }
    }
}

impl<F, B> Default for CommonCell<F, B>
where
    F: From<Rgb24> + DefaultForeground,
    B: From<Rgb24> + DefaultBackground,
{
    fn default() -> Self {
        CommonCell {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: F::default_foreground(),
            background_colour: B::default_background(),
            foreground_depth: 0,
            background_depth: 0,
            access_depth: 0,
        }
    }
}
pub type Iter<'a, C> = grid_2d::GridIter<'a, C>;
pub type IterMut<'a, C> = grid_2d::GridIterMut<'a, C>;
pub type CoordEnumerate<'a, C> = grid_2d::GridEnumerate<'a, C>;

#[derive(Debug, Clone)]
pub struct Grid<F, B>
where
    F: From<Rgb24> + DefaultForeground,
    B: From<Rgb24> + DefaultBackground,
{
    cells: grid_2d::Grid<CommonCell<F, B>>,
}

impl<F, B> Grid<F, B>
where
    F: From<Rgb24> + DefaultForeground + Clone,
    B: From<Rgb24> + DefaultBackground + Clone,
{
    pub fn new(size: Size) -> Self {
        let cells = grid_2d::Grid::new_default(size);

        Self { cells }
    }

    pub fn size(&self) -> Size {
        self.cells.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.cells = grid_2d::Grid::new_default(size);
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Default::default();
        }
    }

    pub fn enumerate(&self) -> CoordEnumerate<CommonCell<F, B>> {
        self.cells.enumerate()
    }

    pub fn iter(&self) -> Iter<CommonCell<F, B>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<CommonCell<F, B>> {
        self.cells.iter_mut()
    }
}

impl<F, B> ViewGrid for Grid<F, B>
where
    F: From<Rgb24> + DefaultForeground,
    B: From<Rgb24> + DefaultBackground,
{
    type Cell = CommonCell<F, B>;
    fn get_mut(&mut self, coord: Coord, depth: i32) -> Option<&mut Self::Cell> {
        if let Some(cell) = self.cells.get_mut(coord) {
            if cell.foreground_depth > depth && cell.background_depth > depth {
                None
            } else {
                cell.access_depth = depth;
                Some(cell)
            }
        } else {
            None
        }
    }
}
