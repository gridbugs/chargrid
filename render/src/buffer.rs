use super::{Blend, Coord, Frame, Rgb24, Size, ViewCell};

#[derive(Debug, Clone, Copy)]
pub struct BufferCell {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground_colour: Rgb24,
    pub background_colour: Rgb24,
    foreground_depth: i8,
    background_depth: i8,
}

impl BufferCell {
    fn set_character(&mut self, character: char, depth: i8) {
        if depth >= self.foreground_depth {
            self.character = character;
            self.foreground_depth = depth;
        }
    }
    fn set_bold(&mut self, bold: bool, depth: i8) {
        if depth >= self.foreground_depth {
            self.bold = bold;
            self.foreground_depth = depth;
        }
    }
    fn set_underline(&mut self, underline: bool, depth: i8) {
        if depth >= self.foreground_depth {
            self.underline = underline;
            self.foreground_depth = depth;
        }
    }
    fn set_foreground_colour(&mut self, colour: Rgb24, depth: i8) {
        if depth >= self.foreground_depth {
            self.foreground_colour = colour;
            self.foreground_depth = depth;
        }
    }
    fn set_background_colour(&mut self, colour: Rgb24, depth: i8) {
        if depth >= self.background_depth {
            self.background_colour = colour;
            self.background_depth = depth;
        }
    }
}

const BLACK: Rgb24 = Rgb24::new_grey(0);
const BLANK_CELL: BufferCell = BufferCell {
    character: ' ',
    bold: false,
    underline: false,
    foreground_colour: BLACK,
    background_colour: BLACK,
    foreground_depth: 0,
    background_depth: 0,
};

pub type BufferIter<'a> = grid_2d::GridIter<'a, BufferCell>;
pub type BufferEnumerate<'a> = grid_2d::GridEnumerate<'a, BufferCell>;
pub type BufferRows<'a> = grid_2d::GridRows<'a, BufferCell>;

#[derive(Debug, Clone)]
pub struct Buffer {
    grid: grid_2d::Grid<BufferCell>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        let grid = grid_2d::Grid::new_copy(size, BLANK_CELL);
        Self { grid }
    }

    pub fn size(&self) -> Size {
        self.grid.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.grid = grid_2d::Grid::new_copy(size, BLANK_CELL);
    }

    pub fn clear(&mut self) {
        for cell in self.grid.iter_mut() {
            *cell = BLANK_CELL;
        }
    }

    pub fn enumerate(&self) -> BufferEnumerate {
        self.grid.enumerate()
    }

    pub fn iter(&self) -> BufferIter {
        self.grid.iter()
    }

    pub fn rows(&self) -> BufferRows {
        self.grid.rows()
    }
}

impl Frame for Buffer {
    fn set_cell_absolute(&mut self, coord: Coord, depth: i8, view_cell: ViewCell) {
        if let Some(cell) = self.grid.get_mut(coord) {
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
    fn blend_cell_background_absolute<B: Blend>(&mut self, coord: Coord, depth: i8, rgb24: Rgb24, alpha: u8, blend: B) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if cell.background_depth <= depth {
                let current_background_colour = cell.background_colour;
                let blended_background_colour = blend.blend(current_background_colour, rgb24, alpha);
                cell.background_colour = blended_background_colour;
                cell.background_depth = depth;
            }
        }
    }
}
