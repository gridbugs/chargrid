use direction::Direction;
use grid_2d::{Coord, Grid, Size};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::mem;

pub type Id = u64;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity<X> {
    id: Id,
    extra: X,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum CharacterTile {
    Player,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    tile: CharacterTile,
}

impl Character {
    pub fn tile(&self) -> CharacterTile {
        self.tile
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wall {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Projectile {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    character: Option<Entity<Character>>,
    wall: Option<Entity<Wall>>,
    projectiles: HashMap<Id, Entity<Projectile>>,
}

impl Cell {
    fn empty() -> Self {
        Self {
            character: None,
            wall: None,
            projectiles: HashMap::new(),
        }
    }
    pub fn is_solid(&self) -> bool {
        self.wall.is_some()
    }
    pub fn character(&self) -> Option<&Entity<Character>> {
        self.character.as_ref()
    }
    pub fn wall(&self) -> Option<&Entity<Wall>> {
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
pub struct GameData {
    coords: HashMap<Id, Coord>,
    grid: Grid<Cell>,
    ids: Ids,
}

pub struct MoveProjectileFailed;

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
        let entity = Entity { id, extra: wall };
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).wall = Some(entity);
    }
    pub fn make_player(&mut self, coord: Coord) -> Id {
        let id = self.ids.next();
        let player = Character {
            tile: CharacterTile::Player,
        };
        let entity = Entity { id, extra: player };
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).character = Some(entity);
        id
    }
    pub fn make_projectile(&mut self, coord: Coord) -> Id {
        let id = self.ids.next();
        let projectile = Projectile {};
        let entity = Entity { id, extra: projectile };
        self.coords.insert(id, coord);
        self.grid.get_checked_mut(coord).projectiles.insert(id, entity);
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
    pub fn move_projectile(&mut self, id: Id, direction: Direction) -> Result<(), MoveProjectileFailed> {
        let projectile_coord = self.coords.get_mut(&id).unwrap();
        let destination_coord = *projectile_coord + direction.coord();
        let (source_cell, destination_cell) = self.grid.get2_checked_mut(*projectile_coord, destination_coord);
        if destination_cell.is_solid() {
            return Err(MoveProjectileFailed);
        }
        let projectile = source_cell.projectiles.remove(&id).unwrap();
        destination_cell.projectiles.insert(id, projectile);
        *projectile_coord = destination_coord;
        Ok(())
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }
    pub fn coords(&self) -> &HashMap<Id, Coord> {
        &self.coords
    }
    pub fn get_cell(&self, coord: Coord) -> Option<&Cell> {
        self.grid.get(coord)
    }
}
