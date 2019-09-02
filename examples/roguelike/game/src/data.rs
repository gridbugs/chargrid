use direction::Direction;
use grid_2d::{Coord, Grid, Size};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::mem;

pub type Id = u64;

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
pub struct Projectile {}

#[derive(Serialize, Deserialize)]
pub struct Cell {
    character: Option<Character>,
    wall: Option<Wall>,
    projectiles: HashMap<Id, Projectile>,
}

impl Cell {
    fn empty() -> Self {
        Self {
            character: None,
            wall: None,
            projectiles: HashMap::new(),
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
    pub fn projectiles(&self) -> impl Iterator<Item = &Projectile> {
        self.projectiles.values()
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
pub struct GameData {
    coords: HashMap<Id, Coord>,
    grid: Grid<Cell>,
    ids: Ids,
}

pub enum ProjectileMoveError {
    DestinationSolid,
}

impl GameData {
    pub fn new(size: Size) -> Self {
        let grid = Grid::new_fn(size, |_| Cell::empty());
        let coords = HashMap::new();
        let ids = Ids::new();
        Self { grid, coords, ids }
    }
    pub fn make_wall(&mut self, coord: Coord) {
        let id = self.ids.next();
        let wall = Wall {};
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).wall = Some(wall);
    }
    pub fn make_player(&mut self, coord: Coord) -> Id {
        let id = self.ids.next();
        let player = Character {
            tile: CharacterTile::Player,
        };
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).character = Some(player);
        id
    }
    pub fn make_projectile(&mut self, coord: Coord) -> Id {
        let id = self.ids.next();
        let projectile = Projectile {};
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).projectiles.insert(id, projectile);
        id
    }
    pub fn move_character(&mut self, id: Id, direction: Direction) {
        let player_coord = self.coords.get_mut(&id).unwrap();
        let destination_coord = *player_coord + direction.coord();
        let (source_cell, destination_cell) = self.grid.get2_checked_mut(*player_coord, destination_coord);
        if destination_cell.is_solid() {
            return;
        }
        mem::swap(&mut source_cell.character, &mut destination_cell.character);
        *player_coord = destination_coord;
    }
    pub fn move_projectile(&mut self, id: Id, destination_coord: Coord) -> Result<(), ProjectileMoveError> {
        let current_coord = self.coords.get_mut(&id).unwrap();
        if *current_coord == destination_coord {
            return Ok(());
        }
        let (source_cell, destination_cell) = self.grid.get2_checked_mut(*current_coord, destination_coord);
        if destination_cell.is_solid() {
            return Err(ProjectileMoveError::DestinationSolid);
        }
        destination_cell
            .projectiles
            .insert(id, source_cell.projectiles.remove(&id).unwrap());
        *current_coord = destination_coord;
        Ok(())
    }
    pub fn remove_projectile(&mut self, id: Id) {
        let current_coord = self.coords.remove(&id).unwrap();
        let current_cell = self.grid.get_checked_mut(current_coord);
        current_cell.projectiles.remove(&id).unwrap();
    }
    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }
    pub fn coords(&self) -> &HashMap<Id, Coord> {
        &self.coords
    }
}