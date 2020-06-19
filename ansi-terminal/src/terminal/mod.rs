use crate::error::Result;
use chargrid_input::*;
use chargrid_render::*;

mod ansi_colour_codes;
mod ansi_terminal;
mod byte_prefix_tree;
mod low_level;
mod term_info_cache;

pub use self::ansi_terminal::{col_encode, AnsiTerminal, ColEncode, DrainInput};

#[derive(Debug, Clone)]
struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Rgb24,
    bg: Rgb24,
    bold: bool,
    underline: bool,
}

impl OutputCell {
    fn matches(&self, cell: &BufferCell) -> bool {
        !self.dirty
            && self.ch == cell.character
            && self.fg == cell.foreground_colour
            && self.bg == cell.background_colour
            && self.bold == cell.bold
            && self.underline == cell.underline
    }
    fn copy_fields(&mut self, cell: &BufferCell) {
        self.dirty = false;
        self.ch = cell.character;
        self.fg = cell.foreground_colour;
        self.bg = cell.background_colour;
        self.bold = cell.bold;
        self.underline = cell.underline;
    }
    fn new() -> Self {
        Self {
            dirty: true,
            ch: ' ',
            fg: Rgb24::new_grey(0),
            bg: Rgb24::new_grey(0),
            bold: false,
            underline: false,
        }
    }
}

pub struct Terminal {
    ansi: AnsiTerminal,
    output_frame: grid_2d::Grid<OutputCell>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let ansi = AnsiTerminal::new()?;
        let size = ansi.size()?;
        let output_frame = grid_2d::Grid::new_fn(size, |_| OutputCell::new());
        Ok(Self { ansi, output_frame })
    }

    pub fn resize_if_necessary(&mut self) -> Result<Size> {
        let size = self.ansi.size()?;
        if size != self.output_frame.size() {
            self.output_frame = grid_2d::Grid::new_fn(size, |_| OutputCell::new());
        }
        Ok(size)
    }

    pub fn size(&self) -> Result<Size> {
        self.ansi.size()
    }

    pub fn draw_frame<E>(&mut self, frame: &mut Buffer) -> Result<()>
    where
        E: ColEncode,
    {
        self.ansi.set_cursor(Coord::new(0, 0))?;
        let mut bold = false;
        let mut underline = false;
        let mut fg = Rgb24::new_grey(0);
        let mut bg = Rgb24::new_grey(0);
        self.ansi.reset();
        self.ansi.clear_underline();
        self.ansi.set_foreground_colour::<E>(fg);
        self.ansi.set_background_colour::<E>(bg);
        let mut must_move_cursor = false;
        for ((coord, cell), output_cell) in frame.enumerate().zip(self.output_frame.iter_mut()) {
            if output_cell.matches(cell) {
                must_move_cursor = true;
                continue;
            }
            let reset = if cell.bold != bold {
                if cell.bold {
                    self.ansi.set_bold();
                    bold = true;
                    false
                } else {
                    self.ansi.reset();
                    bold = false;
                    true
                }
            } else {
                false
            };
            if reset || cell.foreground_colour != fg {
                self.ansi.set_foreground_colour::<E>(cell.foreground_colour);
                fg = cell.foreground_colour;
            }
            if reset || cell.background_colour != bg {
                self.ansi.set_background_colour::<E>(cell.background_colour);
                bg = cell.background_colour;
            }
            if reset || (cell.underline != underline) {
                if cell.underline {
                    self.ansi.set_underline();
                } else {
                    self.ansi.clear_underline();
                }
                underline = cell.underline;
            }
            if must_move_cursor {
                self.ansi.set_cursor(coord)?;
                must_move_cursor = false;
            }
            output_cell.copy_fields(cell);
            self.ansi.add_char_to_buffer(cell.character);
        }
        self.ansi.flush_buffer()?;
        Ok(())
    }

    pub fn drain_input(&mut self) -> Result<DrainInput> {
        self.ansi.drain_input()
    }
}
