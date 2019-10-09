use direction::Direction;
use grid_2d::{Coord, Grid, Size};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

pub trait IdTrait: Hash + Eq + Copy + Debug {
    fn from_u64(u64: u64) -> Self;
    fn to_u64(self) -> u64;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity<T> {
    raw_id: u64,
    data: T,
}

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

pub struct OnePerCellA<'a, T> {
    id_raw: u64,
    coord: &'a mut Coord,
    grid: &'a mut Grid<Option<Entity<T>>>,
}

pub struct OnePerCellB<'a, T> {
    source_coord: &'a mut Coord,
    destination_coord: Coord,
    source_cell: &'a mut Option<Entity<T>>,
    destination_cell: &'a mut Option<Entity<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManyPerCell<I: IdTrait, T> {
    id_allocator: IdAllocator,
    coords: HashMap<I, Coord>,
    grid: Grid<HashMap<I, Entity<T>>>,
}

#[derive(Debug)]
pub enum WithInvalidIdError<E> {
    InvalidIdError,
    Other(E),
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
    pub fn spawn_entity(&mut self, data: T, coord: Coord) -> Result<I, OnePerCellError<I>> {
        let cell = Self::get_empty_cell(&mut self.grid, coord)?;
        let raw_id = self.id_allocator.next();
        let id = I::from_u64(raw_id);
        self.coords.insert(id, coord);
        *cell = Some(Entity { raw_id, data });
        Ok(id)
    }
    fn move_entity_to_coord_inner(
        entity_coord_ref: &mut Coord,
        destination_coord: Coord,
        grid: &mut Grid<Option<Entity<T>>>,
    ) -> Result<(), OnePerCellError<I>> {
        let (source_cell, destination_cell) = grid
            .get2_mut(*entity_coord_ref, destination_coord)
            .or(Err(OnePerCellError::OutOfBounds))?;
        if let Some(entity) = destination_cell.as_ref() {
            return Err(OnePerCellError::CellOccupiedBy(I::from_u64(entity.raw_id)));
        }
        mem::swap(source_cell, destination_cell);
        *entity_coord_ref = destination_coord;
        Ok(())
    }
    pub fn move_entity_to_coord(
        &mut self,
        id: I,
        destination_coord: Coord,
    ) -> Result<(), WithInvalidIdError<OnePerCellError<I>>> {
        let entity_coord_ref = self.coords.get_mut(&id).ok_or(WithInvalidIdError::InvalidIdError)?;
        Self::move_entity_to_coord_inner(entity_coord_ref, destination_coord, &mut self.grid)
            .map_err(WithInvalidIdError::Other)
    }
    pub fn move_entity_in_direction(
        &mut self,
        id: I,
        direction: Direction,
    ) -> Result<(), WithInvalidIdError<OnePerCellError<I>>> {
        let entity_coord_ref = self.coords.get_mut(&id).ok_or(WithInvalidIdError::InvalidIdError)?;
        let destination_coord = *entity_coord_ref + direction.coord();
        Self::move_entity_to_coord_inner(entity_coord_ref, destination_coord, &mut self.grid)
            .map_err(WithInvalidIdError::Other)
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
    pub fn spawn_entity(&mut self, data: T, coord: Coord) -> Result<I, ManyPerCellError> {
        let cell = self.grid.get_mut(coord).ok_or(ManyPerCellError::OutOfBounds)?;
        let raw_id = self.id_allocator.next();
        let id = I::from_u64(raw_id);
        self.coords.insert(id, coord);
        cell.insert(id, Entity { raw_id, data });
        Ok(id)
    }
    fn move_entity_to_coord_inner(
        entity_coord_ref: &mut Coord,
        destination_coord: Coord,
        id: I,
        grid: &mut Grid<HashMap<I, Entity<T>>>,
    ) -> Result<(), ManyPerCellError> {
        let (source_cell, destination_cell) = grid
            .get2_mut(*entity_coord_ref, destination_coord)
            .or(Err(ManyPerCellError::OutOfBounds))?;
        let entity = source_cell.remove(&id).unwrap();
        assert_eq!(I::from_u64(entity.raw_id), id);
        destination_cell.insert(id, entity);
        *entity_coord_ref = destination_coord;
        Ok(())
    }
    pub fn move_entity_to_coord(
        &mut self,
        id: I,
        destination_coord: Coord,
    ) -> Result<(), WithInvalidIdError<ManyPerCellError>> {
        let entity_coord_ref = self.coords.get_mut(&id).ok_or(WithInvalidIdError::InvalidIdError)?;
        Self::move_entity_to_coord_inner(entity_coord_ref, destination_coord, id, &mut self.grid)
            .map_err(WithInvalidIdError::Other)
    }
    pub fn move_entity_in_direction(
        &mut self,
        id: I,
        direction: Direction,
    ) -> Result<(), WithInvalidIdError<ManyPerCellError>> {
        let entity_coord_ref = self.coords.get_mut(&id).ok_or(WithInvalidIdError::InvalidIdError)?;
        let destination_coord = *entity_coord_ref + direction.coord();
        Self::move_entity_to_coord_inner(entity_coord_ref, destination_coord, id, &mut self.grid)
            .map_err(WithInvalidIdError::Other)
    }
}
