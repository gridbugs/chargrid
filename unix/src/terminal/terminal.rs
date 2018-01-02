use std::time::Duration;
use ansi_colour::{Colour, colours};
use error::Result;
use prototty::*;
use super::ansi_terminal::AnsiTerminal;
pub use super::ansi_terminal::DrainInput;

pub const DEFAULT_CH: char = ' ';
pub const DEFAULT_FG: Colour = colours::WHITE;
pub const DEFAULT_BG: Colour = colours::BLACK;

#[derive(Debug, Clone)]
pub struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Colour,
    bg: Colour,
    bold: bool,
    underline: bool,
}

impl Default for OutputCell {
    fn default() -> Self {
        Self {
            dirty: true,
            ch: DEFAULT_CH,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

impl OutputCell {
    pub fn matches(&self, cell: &Cell) -> bool {
        !self.dirty &&
        self.ch == cell.character &&
            self.fg == cell.foreground_colour &&
            self.bg == cell.background_colour &&
            self.bold == cell.bold &&
            self.underline == cell.underline
    }
    pub fn copy_fields(&mut self, cell: &Cell) {
        self.dirty = false;
        self.ch = cell.character;
        self.fg = cell.foreground_colour;
        self.bg = cell.background_colour;
        self.bold = cell.bold;
        self.underline = cell.underline;
    }
}

pub struct Terminal {
    ansi: AnsiTerminal,
    output_grid: Grid<OutputCell>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let ansi = AnsiTerminal::new()?;

        let size = ansi.size()?;
        let output_grid = Grid::new(size);

        Ok(Self {
            ansi,
            output_grid,
        })
    }

    pub fn resize_if_necessary(&mut self) -> Result<Size> {
        let size = self.ansi.size()?;
        if size != self.output_grid.size() {
            self.output_grid.resize(size);
        }
        Ok(size)
    }

    pub fn draw_grid(&mut self, grid: &Grid<Cell>) -> Result<()> {

        self.ansi.set_cursor(Coord::new(0, 0))?;

        let mut bold = false;
        let mut underline = false;
        let mut fg = DEFAULT_FG;
        let mut bg = DEFAULT_BG;
        self.ansi.reset();
        self.ansi.clear_underline();
        self.ansi.set_foreground_colour(fg);
        self.ansi.set_background_colour(bg);

        let mut must_move_cursor = false;

        for ((coord, cell), output_cell) in
            izip!(grid.enumerate(), self.output_grid.iter_mut())
        {
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
                self.ansi.set_foreground_colour(cell.foreground_colour);
                fg = cell.foreground_colour;
            }

            if reset || cell.background_colour != bg {
                self.ansi.set_background_colour(cell.background_colour);
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
