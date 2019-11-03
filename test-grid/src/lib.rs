use grid_2d::{Coord, Grid, Size};
use prototty_render::{Frame, ViewCell};

struct Cell {
    view_cell: Option<ViewCell>,
    depth: i8,
}

impl Cell {
    fn new() -> Self {
        Self {
            view_cell: None,
            depth: 0,
        }
    }
}

pub struct TestGrid {
    grid: Grid<Cell>,
}

impl TestGrid {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_fn(size, |_| Cell::new()),
        }
    }
    pub fn string_rows(&self) -> Vec<String> {
        let mut rows = Vec::new();
        for y in 0..self.grid.height() {
            let mut string = String::new();
            for x in 0..self.grid.width() {
                let cell = self.grid.get_checked(Coord::new(x as i32, y as i32));
                if let Some(view_cell) = cell.view_cell.as_ref() {
                    string.push(view_cell.character.unwrap_or(' '));
                } else {
                    string.push(' ');
                }
            }
            rows.push(string);
        }
        rows
    }
}

impl Frame for TestGrid {
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i8, absolute_cell: ViewCell) {
        if let Some(cell) = self.grid.get_mut(absolute_coord) {
            if absolute_depth >= cell.depth {
                cell.depth = absolute_depth;
                cell.view_cell = Some(absolute_cell);
            }
        }
    }
}
