use prototty_traits::*;
use prototty_grid::*;
use terminal::Terminal;

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
}

impl Renderer for Context {
    type Error = ();
    fn render<V: View<T>, T>(&mut self, view: &V, data: &T) -> Result<(), Self::Error> {
        self.grid.clear();
        view.view(data, Coord::new(0, 0), 0, &mut self.grid);
        self.terminal.draw_grid(&self.grid);
        Ok(())
    }
}
