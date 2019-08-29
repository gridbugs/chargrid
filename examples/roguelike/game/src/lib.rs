pub use direction::Direction;
pub use grid_2d::{Coord, Grid, GridEnumerate, Size};
use hashbrown::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Clone, Copy)]
pub enum Input {
    Move(Direction),
    Fire(Coord),
}

type Id = u64;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum CharacterTile {
    Player,
}

#[derive(Serialize, Deserialize)]
pub struct Character {
    tile: CharacterTile,
}

impl Character {
    pub fn tile(&self) -> CharacterTile {
        self.tile
    }
}

#[derive(Serialize, Deserialize)]
pub struct Wall {}

#[derive(Serialize, Deserialize)]
pub struct Cell {
    character: Option<Character>,
    wall: Option<Wall>,
}

impl Cell {
    fn empty() -> Self {
        Self {
            character: None,
            wall: None,
        }
    }
    fn is_solid(&self) -> bool {
        self.wall.is_some()
    }
    pub fn character(&self) -> Option<&Character> {
        self.character.as_ref()
    }
    pub fn wall(&self) -> Option<&Wall> {
        self.wall.as_ref()
    }
}

#[derive(Serialize, Deserialize)]
struct Ids {
    next: Id,
}

impl Ids {
    fn new() -> Self {
        Self { next: 0 }
    }
    fn next(&mut self) -> Id {
        let next = self.next;
        self.next += 1;
        next
    }
}

#[derive(Serialize, Deserialize)]
struct GameData {
    coords: HashMap<Id, Coord>,
    grid: Grid<Cell>,
    ids: Ids,
}

impl GameData {
    fn new(size: Size) -> Self {
        let grid = Grid::new_fn(size, |_| Cell::empty());
        let coords = HashMap::new();
        let ids = Ids::new();
        Self { grid, coords, ids }
    }
    fn make_wall(&mut self, coord: Coord) {
        let id = self.ids.next();
        let wall = Wall {};
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).wall = Some(wall);
    }
    fn make_player(&mut self, coord: Coord) -> Id {
        let id = self.ids.next();
        let player = Character {
            tile: CharacterTile::Player,
        };
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).character = Some(player);
        id
    }
    fn move_character(&mut self, id: Id, direction: Direction) {
        let player_coord = self.coords.get_mut(&id).unwrap();
        let destination_coord = *player_coord + direction.coord();
        let (source_cell, destination_cell) = self.grid.get2_checked_mut(*player_coord, destination_coord);
        if destination_cell.is_solid() {
            return;
        }
        mem::swap(&mut source_cell.character, &mut destination_cell.character);
        *player_coord = destination_coord;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    data: GameData,
    player_id: Id,
}

impl Game {
    pub fn new<R: Rng>(_rng: &mut R) -> Self {
        let s = include_str!("terrain.txt");
        let rows = s.split("\n").filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
        let mut data = GameData::new(size);
        let mut player_id = None;
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '.' => (),
                    '#' => data.make_wall(coord),
                    '@' => {
                        player_id = Some(data.make_player(coord));
                    }
                    _ => panic!("unexpected char: {}", ch),
                }
            }
        }
        Self {
            data,
            player_id: player_id.expect("didn't create player"),
        }
    }
    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::Move(direction) => self.data.move_character(self.player_id, direction),
            Input::Fire(_) => (),
        }
    }
    pub fn grid(&self) -> &Grid<Cell> {
        &self.data.grid
    }
    pub fn player_coord(&self) -> Coord {
        *self.data.coords.get(&self.player_id).unwrap()
    }
}
