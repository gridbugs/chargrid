pub use super::ansi_terminal::DrainInput;
use super::ansi_terminal::{AnsiTerminal, ColEncode};
use error::Result;
use prototty_grid::*;
use prototty_input::*;
use prototty_render::*;
use std::time::Duration;

type Cell = CommonCell<Rgb24>;

#[derive(Debug, Clone)]
pub struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Rgb24,
    bg: Rgb24,
    bold: bool,
    underline: bool,
}

impl OutputCell {
    pub fn matches(&self, cell: &Cell) -> bool {
        !self.dirty
            && self.ch == cell.character
            && self.fg == cell.foreground_colour
            && self.bg == cell.background_colour
            && self.bold == cell.bold
            && self.underline == cell.underline
    }
    pub fn copy_fields(&mut self, cell: &Cell) {
        self.dirty = false;
        self.ch = cell.character;
        self.fg = cell.foreground_colour;
        self.bg = cell.background_colour;
        self.bold = cell.bold;
        self.underline = cell.underline;
    }
    fn new_default(fg: Rgb24, bg: Rgb24) -> Self {
        Self {
            dirty: true,
            ch: ' ',
            fg,
            bg,
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
    pub fn new(fg: Rgb24, bg: Rgb24) -> Result<Self> {
        let ansi = AnsiTerminal::new()?;
        let size = ansi.size()?;
        let output_frame = grid_2d::Grid::new_fn(size, |_| OutputCell::new_default(fg, bg));
        Ok(Self { ansi, output_frame })
    }

    pub fn resize_if_necessary(&mut self, fg: Rgb24, bg: Rgb24) -> Result<Size> {
        let size = self.ansi.size()?;
        if size != self.output_frame.size() {
            self.output_frame = grid_2d::Grid::new_fn(size, |_| OutputCell::new_default(fg, bg));
        }
        Ok(size)
    }

    pub fn size(&self) -> Result<Size> {
        self.ansi.size()
    }

    pub fn draw_frame<C, E>(&mut self, frame: &mut Grid<C>) -> Result<()>
    where
        C: ColourConversion<Colour = Rgb24>,
        E: ColEncode,
    {
        self.ansi.set_cursor(Coord::new(0, 0))?;
        let mut bold = false;
        let mut underline = false;
        let mut fg = frame.default_foreground();
        let mut bg = frame.default_background();
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

    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.ansi.poll_input()
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        self.ansi.wait_input()
    }

    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.ansi.wait_input_timeout(timeout)
    }
}
