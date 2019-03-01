pub extern crate grid_2d;
extern crate prototty_render;

use prototty_render::*;

pub trait FrontendColour {
    fn default_foreground() -> Self;
    fn default_background() -> Self;
    fn from_rgb24(rgb24: Rgb24) -> Self;
}

#[derive(Debug, Clone)]
pub struct CommonCell<C> {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: C,
    pub background_colour: C,
    foreground_depth: i32,
    background_depth: i32,
}

impl<C: FrontendColour> CommonCell<C> {
    fn set_character(&mut self, character: char, depth: i32) {
        if depth >= self.foreground_depth {
            self.character = character;
            self.foreground_depth = depth;
        }
    }
    fn set_bold(&mut self, bold: bool, depth: i32) {
        if depth >= self.foreground_depth {
            self.bold = bold;
            self.foreground_depth = depth;
        }
    }
    fn set_underline(&mut self, underline: bool, depth: i32) {
        if depth >= self.foreground_depth {
            self.underline = underline;
            self.foreground_depth = depth;
        }
    }
    fn set_foreground_colour(&mut self, colour: Rgb24, depth: i32) {
        if depth >= self.foreground_depth {
            self.foreground_colour = C::from_rgb24(colour);
            self.foreground_depth = depth;
        }
    }
    fn set_background_colour(&mut self, colour: Rgb24, depth: i32) {
        if depth >= self.background_depth {
            self.background_colour = C::from_rgb24(colour);
            self.background_depth = depth;
        }
    }
}

impl<C: FrontendColour> Default for CommonCell<C> {
    fn default() -> Self {
        CommonCell {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: C::default_foreground(),
            background_colour: C::default_background(),
            foreground_depth: 0,
            background_depth: 0,
        }
    }
}
pub type Iter<'a, C> = grid_2d::GridIter<'a, C>;
pub type IterMut<'a, C> = grid_2d::GridIterMut<'a, C>;
pub type Enumerate<'a, C> = grid_2d::GridEnumerate<'a, C>;

#[derive(Debug, Clone)]
pub struct Grid<C> {
    cells: grid_2d::Grid<CommonCell<C>>,
}

impl<C: FrontendColour> Grid<C> {
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

    pub fn enumerate(&self) -> Enumerate<CommonCell<C>> {
        self.cells.enumerate()
    }

    pub fn iter(&self) -> Iter<CommonCell<C>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<CommonCell<C>> {
        self.cells.iter_mut()
    }
}

impl<C: FrontendColour> ViewGrid for Grid<C> {
    fn set_cell(&mut self, coord: Coord, depth: i32, info: ViewCell) {
        if let Some(cell) = self.cells.get_mut(coord) {
            if cell.foreground_depth <= depth || cell.background_depth <= depth {
                if let Some(character) = info.character {
                    cell.set_character(character, depth);
                }
                if let Some(bold) = info.bold {
                    cell.set_bold(bold, depth);
                }
                if let Some(underline) = info.underline {
                    cell.set_underline(underline, depth);
                }
                if let Some(foreground) = info.foreground {
                    cell.set_foreground_colour(foreground, depth);
                }
                if let Some(background) = info.background {
                    cell.set_background_colour(background, depth);
                }
            }
        }
    }
    fn size(&self) -> Size {
        self.cells.size()
    }
}
