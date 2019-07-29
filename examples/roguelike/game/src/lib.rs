pub use direction::Direction;
pub use grid_2d::{Coord, Grid, Size};

#[derive(Clone)]
pub enum Occupant {
    Player,
    Wall,
}

#[derive(Clone)]
pub struct Cell {
    pub occupant: Option<Occupant>,
}

pub struct Player {
    pub coord: Coord,
}

impl Cell {
    pub fn empty() -> Self {
        Self { occupant: None }
    }
}

pub struct Game {
    grid: Grid<Cell>,
    player: Player,
}

pub enum Input {
    Move(Direction),
}

impl Game {
    pub fn new() -> Self {
        let s = include_str!("terrain.txt");
        let rows = s.split("\n").filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
        let mut grid = Grid::new_clone(size, Cell::empty());
        let mut player = Player {
            coord: Coord::new(0, 0),
        };
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let mut cell = grid.get_checked_mut(coord);
                match ch {
                    '.' => cell.occupant = None,
                    '#' => cell.occupant = Some(Occupant::Wall),
                    '@' => {
                        cell.occupant = Some(Occupant::Player);
                        player.coord = coord;
                    }
                    _ => panic!("unexpected char: {}", ch),
                }
            }
        }
        Self { grid, player }
    }

    pub fn to_render(&self) -> ToRender {
        ToRender { grid: &self.grid }
    }

    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::Move(direction) => {
                let player_next_coord = self.player.coord + direction.coord();
                if let Some(cell) = self.grid.get_mut(player_next_coord) {
                    if cell.occupant.is_none() {
                        cell.occupant = Some(Occupant::Player);
                        self.grid.get_checked_mut(self.player.coord).occupant = None;
                        self.player.coord = player_next_coord;
                    }
                }
            }
        }
    }
}

pub struct ToRender<'a> {
    pub grid: &'a Grid<Cell>,
}
