use std::time::Duration;
use cgmath::Vector2;
use ansi_colour::Colour;
use core::terminal::Terminal;
use input::Input;
use error::Result;
use defaults::*;
use grid::*;
use view::*;

pub struct Context {
    terminal: Terminal,
    grid: Grid<Cell>,
    output_grid: Grid<OutputCell>,
    default_fg: Colour,
    default_bg: Colour,
}

impl Context {
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    pub fn from_terminal(terminal: Terminal) -> Result<Self> {

        let size = terminal.size()?;
        let grid = Grid::new(size);
        let output_grid = Grid::new(size);

        Ok(Self {
            terminal,
            output_grid,
            grid,
            default_fg: DEFAULT_FG,
            default_bg: DEFAULT_BG,
        })
    }

    pub fn set_default_foreground_colour(&mut self, colour: Colour) {
        self.default_fg = colour;
    }

    pub fn set_default_background_colour(&mut self, colour: Colour) {
        self.default_bg = colour;
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.size()?;
        if size != self.grid.size() {
            self.grid.resize(size);
            self.output_grid.resize(size);
        }

        Ok(())
    }

    pub fn render<V: View>(&mut self, view: &V) -> Result<()> {
        self.resize_if_necessary()?;

        self.grid.clear();
        view.view(Vector2::new(0, 0), 0, &mut self.grid);
        self.send_grid_contents()?;
        self.terminal.flush_buffer()?;

        Ok(())
    }

    fn send_grid_contents(&mut self) -> Result<()> {

        self.terminal.set_cursor(Vector2::new(0, 0))?;

        let mut bold = false;
        let mut underline = false;
        let mut fg = self.default_fg;
        let mut bg = self.default_bg;
        self.terminal.set_foreground_colour(fg);
        self.terminal.set_background_colour(bg);

        let mut must_move_cursor = false;

        for ((coord, cell), output_cell) in
            izip!(self.grid.enumerate(), self.output_grid.iter_mut())
        {
            if output_cell.matches(cell) {
                must_move_cursor = true;
                continue;
            }

            let reset = if cell.bold != bold {
                if cell.bold {
                    self.terminal.set_bold();
                    bold = true;
                    false
                } else {
                    self.terminal.reset();
                    bold = false;
                    true
                }
            } else {
                false
            };

            if reset || cell.fg != fg {
                self.terminal.set_foreground_colour(cell.fg);
                fg = cell.fg;
            }

            if reset || cell.bg != bg {
                self.terminal.set_background_colour(cell.bg);
                bg = cell.bg;
            }

            if reset || (cell.underline != underline) {
                if cell.underline {
                    self.terminal.set_underline();
                } else {
                    self.terminal.clear_underline();
                }
                underline = cell.underline;
            }

            if must_move_cursor {
                self.terminal.set_cursor(coord.cast())?;
                must_move_cursor = false;
            }

            output_cell.copy_fields(cell);
            self.terminal.add_char_to_buffer(cell.ch);
        }

        Ok(())
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        self.terminal.wait_input()
    }
    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.terminal.wait_input_timeout(timeout)
    }
    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.terminal.poll_input()
    }
}
