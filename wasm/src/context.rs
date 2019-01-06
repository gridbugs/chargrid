use cell::Colour;
use prototty_grid::*;
use prototty_render::*;
use terminal::Terminal;

pub struct Context {
    terminal: Terminal,
    grid: Grid<Colour, Colour>,
}

impl Context {
    pub fn new() -> Self {
        let terminal = Terminal::new();
        let grid = Grid::new(terminal.size());
        Self { terminal, grid }
    }

    pub fn quit(&self) {
        self.terminal.quit();
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) {
        self.render_at(view, data, Coord::new(0, 0), 0)
    }

    pub fn render_at<V: View<T>, T>(&mut self, view: &mut V, data: &T, offset: Coord, depth: i32) {
        self.grid.clear();
        view.view(data, offset, depth, &mut self.grid);
        self.terminal.draw_grid(&self.grid);
    }

    pub fn size(&self) -> Size {
        self.terminal.size()
    }
}
