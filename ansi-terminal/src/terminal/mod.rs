use crate::error::Result;
use chargrid_input::*;
use chargrid_runtime::{rgb_int::Rgb24, FrameBuffer, FrameBufferCell, Rgba32, Size};

mod ansi_colour_codes;
mod ansi_terminal;
mod byte_prefix_tree;
mod low_level;
mod term_info_cache;

pub use self::ansi_terminal::{col_encode, AnsiTerminal, ColEncode, DrainInput};

fn rgba32_to_rgb24(Rgba32 { r, g, b, a: _ }: Rgba32) -> Rgb24 {
    Rgb24 { r, g, b }
}

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
        self.ansi.set_foreground_colour::<E>(rgba32_to_rgb24(fg));
        self.ansi.set_background_colour::<E>(rgba32_to_rgb24(bg));
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
                    .set_foreground_colour::<E>(rgba32_to_rgb24(cell.foreground));
                fg = cell.foreground;
            }
            if reset || cell.background != bg {
                self.ansi
                    .set_background_colour::<E>(rgba32_to_rgb24(cell.background));
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
