use std::time::Duration;
use cgmath::Vector2;
use core::terminal::Terminal;
use input::Input;
use error::Result;
use defaults::*;
use grid::*;
use view::*;

/**
 * An interface to a terminal for rendering `View`s, and getting input.
 */
pub struct Context {
    terminal: Terminal,
    grid: Grid<Cell>,
    output_grid: Grid<OutputCell>,
}

impl Context {
    /**
     * Initialise a new context using the current terminal.
     */
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    fn from_terminal(terminal: Terminal) -> Result<Self> {

        let size = terminal.size()?;
        let grid = Grid::new(size);
        let output_grid = Grid::new(size);

        Ok(Self {
            terminal,
            output_grid,
            grid,
        })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.size()?;
        if size != self.grid.size() {
            self.grid.resize(size);
            self.output_grid.resize(size);
        }

        Ok(())
    }

    /**
     * Clear the screen, and render the given `View` starting in the top-left corner.
     */
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
        let mut fg = DEFAULT_FG;
        let mut bg = DEFAULT_BG;
        self.terminal.reset();
        self.terminal.clear_underline();
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

    /**
     * Gets an input event from the terminal if one is present,
     * returning immediately.
     */
    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.terminal.poll_input()
    }

    /**
     * Gets an input event from the terminal, waiting until
     * an event occurs.
     */
    pub fn wait_input(&mut self) -> Result<Input> {
        self.terminal.wait_input()
    }

    /**
     * Gets an input event from the terminal, waiting until
     * either an event occurs, or the timeout expires, in which
     * case this method returns `None`.
     */
    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.terminal.wait_input_timeout(timeout)
    }
}
