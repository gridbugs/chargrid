use grid_2d::{Coord, Grid, Size};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

pub trait IdTrait: Hash + Eq + Copy + Debug {
    fn from_u64(u64: u64) -> Self;
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct WallId(u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct CharacterId(u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ProjectileId(u64);

impl IdTrait for WallId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
}

impl IdTrait for CharacterId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
}

impl IdTrait for ProjectileId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity<T> {
    raw_id: u64,
    data: T,
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
struct IdAllocator {
    next: u64,
}

impl IdAllocator {
    fn new() -> Self {
        Self { next: 0 }
    }
    fn next(&mut self) -> u64 {
        let next = self.next;
        self.next += 1;
        next
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnePerCell<I: IdTrait, T> {
    id_allocator: IdAllocator,
    coords: HashMap<I, Coord>,
    grid: Grid<Option<Entity<T>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManyPerCell<I: IdTrait, T> {
    id_allocator: IdAllocator,
    coords: HashMap<I, Coord>,
    grid: Grid<HashMap<I, Entity<T>>>,
}

#[derive(Debug)]
pub enum OnePerCellError<I> {
    OutOfBounds,
    CellOccupiedBy(I),
}

#[derive(Debug)]
pub enum ManyPerCellError {
    OutOfBounds,
}

impl<I: IdTrait, T> OnePerCell<I, T> {
    pub fn new(size: Size) -> Self {
        let id_allocator = IdAllocator::new();
        let coords = HashMap::new();
        let grid = Grid::new_fn(size, |_| None);
        Self {
            id_allocator,
            coords,
            grid,
        }
    }
    fn get_empty_cell(
        grid: &mut Grid<Option<Entity<T>>>,
        coord: Coord,
    ) -> Result<&mut Option<Entity<T>>, OnePerCellError<I>> {
        let cell = grid.get_mut(coord).ok_or(OnePerCellError::OutOfBounds)?;
        if let Some(entity) = cell.as_ref() {
            return Err(OnePerCellError::CellOccupiedBy(I::from_u64(entity.raw_id)));
        }
        Ok(cell)
    }
    pub fn spawn(&mut self, data: T, coord: Coord) -> Result<I, OnePerCellError<I>> {
        let cell = Self::get_empty_cell(&mut self.grid, coord)?;
        let raw_id = self.id_allocator.next();
        let id = I::from_u64(raw_id);
        self.coords.insert(id, coord);
        *cell = Some(Entity { raw_id, data });
        Ok(id)
    }
    pub fn move_entity(&mut self, id: I, destination_coord: Coord) -> Result<(), OnePerCellError<I>> {
        let entity_coord_ref = self.coords.get_mut(&id).unwrap();
        let (source_cell, destination_cell) = self
            .grid
            .get2_mut(*entity_coord_ref, destination_coord)
            .or(Err(OnePerCellError::OutOfBounds))?;
        if let Some(entity) = destination_cell.as_ref() {
            return Err(OnePerCellError::CellOccupiedBy(I::from_u64(entity.raw_id)));
        }
        assert_eq!(I::from_u64(source_cell.as_ref().unwrap().raw_id), id);
        mem::swap(source_cell, destination_cell);
        *entity_coord_ref = destination_coord;
        Ok(())
    }
}

impl<I: IdTrait, T> ManyPerCell<I, T> {
    pub fn new(size: Size) -> Self {
        let id_allocator = IdAllocator::new();
        let coords = HashMap::new();
        let grid = Grid::new_fn(size, |_| HashMap::new());
        Self {
            id_allocator,
            coords,
            grid,
        }
    }
    pub fn spawn(&mut self, data: T, coord: Coord) -> Result<I, ManyPerCellError> {
        let cell = self.grid.get_mut(coord).ok_or(ManyPerCellError::OutOfBounds)?;
        let raw_id = self.id_allocator.next();
        let id = I::from_u64(raw_id);
        self.coords.insert(id, coord);
        cell.insert(id, Entity { raw_id, data });
        Ok(id)
    }
    pub fn move_entity(&mut self, id: I, destination_coord: Coord) -> Result<(), ManyPerCellError> {
        let entity_coord_ref = self.coords.get_mut(&id).unwrap();
        let (source_cell, destination_cell) = self
            .grid
            .get2_mut(*entity_coord_ref, destination_coord)
            .or(Err(ManyPerCellError::OutOfBounds))?;
        let entity = source_cell.remove(&id).unwrap();
        assert_eq!(I::from_u64(entity.raw_id), id);
        destination_cell.insert(id, entity);
        *entity_coord_ref = destination_coord;
        Ok(())
    }
}

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct GameData {
    ids: Ids,
    entity_coords: HashMap<Id, Coord>,
    characters: Grid<Option<Entity<Character>>>,
    walls: Grid<Option<Entity<Wall>>>,
    projectiles: Grid<HashMap<Id, Entity<Projectile>>>,
}


#[derive(Debug)]
pub enum OnePerCellSpawnError {
    OutOfBounds,
    CellOccupiedBy(Id),
}

#[derive(Debug)]
pub struct OutOfBounds;

fn get_empty_option_cell<T>(
    grid: &mut Grid<Option<Entity<T>>>,
    coord: Coord,
) -> Result<&mut Option<Entity<T>>, OnePerCellSpawnError> {
    let cell = grid.get_mut(coord).ok_or(OnePerCellSpawnError::OutOfBounds)?;
    if let Some(entity) = cell.as_ref() {
        return Err(OnePerCellSpawnError::CellOccupiedBy(entity.id));
    }
    Ok(cell)
}

impl GameData {
    pub fn new(size: Size) -> Self {
        let ids = Ids::new();
        let entity_coords = HashMap::new();
        let characters = Grid::new_fn(size, |_| None);
        let walls = Grid::new_fn(size, |_| None);
        let projectiles = Grid::new_fn(size, |_| HashMap::new());
        Self {
            ids,
            entity_coords,
            characters,
            walls,
            projectiles,
        }
    }
    pub fn spawn_wall(&mut self, coord: Coord) -> Result<Id, OnePerCellSpawnError> {
        let cell = get_empty_option_cell(&mut self.walls, coord)?;
        let id = self.ids.next();
        self.entity_coords.insert(id, coord);
        *cell = Some(Entity { id, extra: Wall {} });
        Ok(id)
    }
    pub fn spawn_player(&mut self, coord: Coord) -> Result<Id, OnePerCellSpawnError> {
        let cell = get_empty_option_cell(&mut self.characters, coord)?;
        let id = self.ids.next();
        self.entity_coords.insert(id, coord);
        *cell = Some(Entity {
            id,
            extra: Character {
                tile: CharacterTile::Player,
            },
        });
        Ok(id)
    }
    pub fn spawn_projectile(&mut self, coord: Coord) -> Result<Id, OutOfBounds> {
        let cell = self.projectiles.get_mut(coord).ok_or(OutOfBounds)?;
        let id = self.ids.next();
        self.entity_coords.insert(id, coord);
        cell.insert(
            id,
            Entity {
                id,
                extra: Projectile {},
            },
        );
        Ok(id)
    }
}*/
