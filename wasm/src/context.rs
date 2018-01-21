use prototty::*;
use prototty_grid::*;
use terminal::Terminal;
use cell::Cell;

pub struct Context {
    terminal: Terminal,
    grid: Grid<Cell>,
}

impl Context {
    pub fn new() -> Self {
        let terminal = Terminal::new();
        let grid = Grid::new(terminal.size());
        Self {
            terminal,
            grid,
        }
    }

    pub fn quit(&self) {
        self.terminal.quit();
    }
}

impl Renderer for Context {
    type Error = ();
    fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) -> Result<(), Self::Error> {
        self.grid.clear();
        view.view(data, Coord::new(0, 0), 0, &mut self.grid);
        self.terminal.draw_grid(&self.grid);
        Ok(())
    }
}
