use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use cgmath::Vector2;
use core::terminal::Terminal;
use input::Input;
use error::Result;
use grid::*;
use defaults::*;
use elements::*;

pub struct Context {
    terminal: Terminal,
    seq: u64,
    grid: Grid<Cell>,
    output_grid: Grid<OutputCell>,
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
            seq: 0,
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

    pub fn render(&mut self, root: &ElementHandle) -> Result<()> {
        self.resize_if_necessary()?;
        self.seq += 1;

        self.grid.clear();
        root.render(&mut self.grid, self.seq, Vector2::new(0, 0), 0);
        self.send_grid_contents()?;
        self.terminal.flush_buffer()?;

        Ok(())
    }

    fn send_grid_contents(&mut self) -> Result<()> {

        let mut fg = DEFAULT_FG;
        let mut bg = DEFAULT_BG;
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

            if must_move_cursor {
                self.terminal.set_cursor(coord.cast())?;
                must_move_cursor = false;
            }

            if cell.fg != fg {
                self.terminal.set_foreground_colour(cell.fg);
                fg = cell.fg;
            }

            if cell.bg != bg {
                self.terminal.set_background_colour(cell.bg);
                bg = cell.bg;
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
}

pub(crate) type ElementCell<T> = Rc<RefCell<T>>;
pub(crate) fn element_cell<T>(t: T) -> ElementCell<T> {
    Rc::new(RefCell::new(t))
}

#[derive(Debug, Clone)]
pub enum ElementHandle {
    AbsDiv(AbsDiv),
    Text(Text),
    Canvas(Canvas),
    BorderContainer(BorderContainer),
}

impl ElementHandle {
    pub fn abs_div(&self) -> Option<&AbsDiv> {
        if let &ElementHandle::AbsDiv(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn text(&self) -> Option<&Text> {
        if let &ElementHandle::Text(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn canvas(&self) -> Option<&Canvas> {
        if let &ElementHandle::Canvas(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        match self {
            &ElementHandle::AbsDiv(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::Text(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::Canvas(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::BorderContainer(ref e) => e.render(grid, seq, offset, depth),
        }
    }
    pub(crate) fn size(&self) -> Vector2<u16> {
        match self {
            &ElementHandle::AbsDiv(ref e) => e.size(),
            &ElementHandle::Text(ref e) => e.size(),
            &ElementHandle::Canvas(ref e) => e.size(),
            &ElementHandle::BorderContainer(ref e) => e.size(),
        }
    }
}

impl From<AbsDiv> for ElementHandle {
    fn from(e: AbsDiv) -> Self {
        ElementHandle::AbsDiv(e)
    }
}
impl From<Text> for ElementHandle {
    fn from(e: Text) -> Self {
        ElementHandle::Text(e)
    }
}
impl From<Canvas> for ElementHandle {
    fn from(e: Canvas) -> Self {
        ElementHandle::Canvas(e)
    }
}
impl From<BorderContainer> for ElementHandle {
    fn from(e: BorderContainer) -> Self {
        ElementHandle::BorderContainer(e)
    }
}
