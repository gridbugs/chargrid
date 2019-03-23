pub extern crate grid_2d;
extern crate prototty_render;

use prototty_render::*;

pub trait ColourConversion {
    type Colour;
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour;
    fn default_foreground(&mut self) -> Self::Colour;
    fn default_background(&mut self) -> Self::Colour;
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

impl<C> CommonCell<C> {
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
    fn set_foreground_colour(&mut self, colour: C, depth: i32) {
        if depth >= self.foreground_depth {
            self.foreground_colour = colour;
            self.foreground_depth = depth;
        }
    }
    fn set_background_colour(&mut self, colour: C, depth: i32) {
        if depth >= self.background_depth {
            self.background_colour = colour;
            self.background_depth = depth;
        }
    }
    fn new_default(foreground_colour: C, background_colour: C) -> Self {
        Self {
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour,
            background_colour,
            foreground_depth: 0,
            background_depth: 0,
        }
    }
}
pub type Iter<'a, C> = grid_2d::GridIter<'a, C>;
pub type IterMut<'a, C> = grid_2d::GridIterMut<'a, C>;
pub type Enumerate<'a, C> = grid_2d::GridEnumerate<'a, C>;

#[derive(Debug, Clone)]
pub struct Grid<C: ColourConversion> {
    cells: grid_2d::Grid<CommonCell<C::Colour>>,
    colour_conversion: C,
}

impl<C: ColourConversion> Grid<C> {
    pub fn new(size: Size, mut colour_conversion: C) -> Self {
        let cells = grid_2d::Grid::new_fn(size, |_| {
            let foreground = colour_conversion.default_foreground();
            let background = colour_conversion.default_background();
            CommonCell::new_default(foreground, background)
        });
        Self {
            cells,
            colour_conversion,
        }
    }

    pub fn size(&self) -> Size {
        self.cells.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.cells = grid_2d::Grid::new_fn(size, |_| {
            let foreground = self.colour_conversion.default_foreground();
            let background = self.colour_conversion.default_background();
            CommonCell::new_default(foreground, background)
        });
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            let foreground = self.colour_conversion.default_foreground();
            let background = self.colour_conversion.default_background();
            *cell = CommonCell::new_default(foreground, background);
        }
    }

    pub fn enumerate(&self) -> Enumerate<CommonCell<C::Colour>> {
        self.cells.enumerate()
    }

    pub fn iter(&self) -> Iter<CommonCell<C::Colour>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<CommonCell<C::Colour>> {
        self.cells.iter_mut()
    }
}

impl<C: ColourConversion> ViewGrid for Grid<C> {
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
                    cell.set_foreground_colour(
                        self.colour_conversion.convert_rgb24(foreground),
                        depth,
                    );
                }
                if let Some(background) = view_cell.background() {
                    cell.set_background_colour(
                        self.colour_conversion.convert_rgb24(background),
                        depth,
                    );
                }
            }
        }
    }
    fn size(&self) -> Size {
        self.cells.size()
    }
}
