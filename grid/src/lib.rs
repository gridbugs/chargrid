pub extern crate grid_2d;
extern crate prototty_render;

use prototty_render::*;

#[derive(Debug, Clone)]
pub struct CommonCell {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: Rgb24,
    pub background_colour: Rgb24,
    foreground_depth: i32,
    background_depth: i32,
}

impl CommonCell {
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
            self.foreground_colour = colour;
            self.foreground_depth = depth;
        }
    }
    fn set_background_colour(&mut self, colour: Rgb24, depth: i32) {
        if depth >= self.background_depth {
            self.background_colour = colour;
            self.background_depth = depth;
        }
    }
    fn new() -> Self {
        const BLACK: Rgb24 = Rgb24::new_grey(0);
        Self {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: BLACK,
            background_colour: BLACK,
            foreground_depth: 0,
            background_depth: 0,
        }
    }
}
pub type Iter<'a> = grid_2d::GridIter<'a, CommonCell>;
pub type IterMut<'a> = grid_2d::GridIterMut<'a, CommonCell>;
pub type Enumerate<'a> = grid_2d::GridEnumerate<'a, CommonCell>;

#[derive(Debug, Clone)]
pub struct Grid {
    cells: grid_2d::Grid<CommonCell>,
}

impl Grid {
    pub fn new(size: Size) -> Self {
        let cells = grid_2d::Grid::new_fn(size, |_| CommonCell::new());
        Self { cells }
    }

    pub fn size(&self) -> Size {
        self.cells.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.cells = grid_2d::Grid::new_fn(size, |_| CommonCell::new());
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = CommonCell::new();
        }
    }

    pub fn enumerate(&self) -> Enumerate {
        self.cells.enumerate()
    }

    pub fn iter(&self) -> Iter {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut {
        self.cells.iter_mut()
    }
}

impl Frame for Grid {
    fn set_cell_absolute(&mut self, coord: Coord, depth: i32, view_cell: ViewCell) {
        if let Some(cell) = self.cells.get_mut(coord) {
            if cell.foreground_depth <= depth || cell.background_depth <= depth {
                if let Some(character) = view_cell.character() {
                    cell.set_character(character, depth);
                }
                if let Some(bold) = view_cell.bold() {
                    cell.set_bold(bold, depth);
                }
                if let Some(underline) = view_cell.underline() {
                    cell.set_underline(underline, depth);
                }
                if let Some(foreground) = view_cell.foreground() {
                    cell.set_foreground_colour(foreground, depth);
                }
                if let Some(background) = view_cell.background() {
                    cell.set_background_colour(background, depth);
                }
            }
        }
    }
}
