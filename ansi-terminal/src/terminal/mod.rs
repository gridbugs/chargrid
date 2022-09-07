use crate::error::Result;
use chargrid_input::*;
use chargrid_runtime::{FrameBuffer, FrameBufferCell, Rgba32, Size};

mod ansi_colour_codes;
mod ansi_terminal;
mod byte_prefix_tree;
mod term_info_cache;

#[cfg(unix)]
mod low_level;

#[cfg(not(unix))]
mod low_level {
    use crate::error::{Error, Result};
    use chargrid_runtime::Size;
    use std::io;

    pub struct LowLevel {}

    impl LowLevel {
        pub fn new() -> Result<Self> {
            Err(Error::NonUnixOS)
        }

        pub fn send(&mut self, _data: &str) -> io::Result<()> {
            panic!("Unimplemented on non-unix OS")
        }

        pub fn read_polling(&mut self, _buf: &mut Vec<u8>) -> Result<()> {
            panic!("Unimplemented on non-unix OS")
        }

        pub fn size(&self) -> Result<Size> {
            panic!("Unimplemented on non-unix OS")
        }
    }
}

pub use self::ansi_terminal::{col_encode, AnsiTerminal, ColEncode, DrainInput};

#[derive(Debug, Clone)]
struct OutputCell {
    dirty: bool,
    ch: char,
    fg: Rgba32,
    bg: Rgba32,
    bold: bool,
    underline: bool,
}

impl OutputCell {
    fn matches(&self, cell: &FrameBufferCell) -> bool {
        !self.dirty
            && self.ch == cell.character
            && self.fg == cell.foreground
            && self.bg == cell.background
            && self.bold == cell.bold
            && self.underline == cell.underline
    }
    fn copy_fields(&mut self, cell: &FrameBufferCell) {
        self.dirty = false;
        self.ch = cell.character;
        self.fg = cell.foreground;
        self.bg = cell.background;
        self.bold = cell.bold;
        self.underline = cell.underline;
    }
    fn new() -> Self {
        Self {
            dirty: true,
            ch: ' ',
            fg: Rgba32::new_grey(0),
            bg: Rgba32::new_grey(0),
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

    pub fn draw_frame<E>(&mut self, frame: &mut FrameBuffer) -> Result<()>
    where
        E: ColEncode,
    {
        self.ansi.set_cursor(Coord::new(0, 0))?;
        let mut bold = false;
        let mut underline = false;
        let mut fg = Rgba32::new_grey(0);
        let mut bg = Rgba32::new_grey(0);
        self.ansi.reset();
        self.ansi.clear_underline();
        self.ansi.set_foreground_colour::<E>(fg.to_rgb24());
        self.ansi.set_background_colour::<E>(bg.to_rgb24());
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
            if reset || cell.foreground != fg {
                self.ansi
                    .set_foreground_colour::<E>(cell.foreground.to_rgb24());
                fg = cell.foreground;
            }
            if reset || cell.background != bg {
                self.ansi
                    .set_background_colour::<E>(cell.background.to_rgb24());
                bg = cell.background;
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
