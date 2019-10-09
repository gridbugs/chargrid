#![allow(dead_code)]
use grid_2d::{Coord, Get2Error, Grid, Size};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

pub type RawId = u64;

pub trait IdTrait: Hash + Eq + Copy + Debug {
    fn from_raw(raw_id: RawId) -> Self;
    fn to_raw(self) -> RawId;
}

#[derive(Serialize, Deserialize, Debug)]
struct Cell<E> {
    raw_id: RawId,
    entity: E,
}

#[derive(Serialize, Deserialize, Debug)]
struct RawIdAllocator {
    next: RawId,
}

impl RawIdAllocator {
    fn new() -> Self {
        Self { next: 0 }
    }
    fn next(&mut self) -> RawId {
        let next = self.next;
        self.next += 1;
        next
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityGrid<I: IdTrait, E> {
    raw_id_allocator: RawIdAllocator,
    coords: HashMap<I, Coord>,
    grid: Grid<Option<Cell<E>>>,
}

impl<I: IdTrait, E> EntityGrid<I, E> {
    pub fn new(size: Size) -> Self {
        let raw_id_allocator = RawIdAllocator::new();
        let coords = HashMap::new();
        let grid = Grid::new_fn(size, |_| None);
        Self {
            raw_id_allocator,
            coords,
            grid,
        }
    }
    fn get_empty_cell(
        grid: &mut Grid<Option<Cell<E>>>,
        coord: Coord,
    ) -> Result<&mut Option<Cell<E>>, EmptyCellError<I>> {
        let cell = grid.get_mut(coord).ok_or(EmptyCellError::OutOfBounds)?;
        if let Some(entity) = cell.as_ref() {
            return Err(EmptyCellError::OccupiedBy(I::from_raw(entity.raw_id)));
        }
        Ok(cell)
    }
    pub fn spawn_entity(&mut self, coord: Coord, entity: E) -> Result<I, EmptyCellError<I>> {
        let cell = Self::get_empty_cell(&mut self.grid, coord)?;
        let raw_id = self.raw_id_allocator.next();
        let id = I::from_raw(raw_id);
        self.coords.insert(id, coord);
        *cell = Some(Cell { raw_id, entity });
        Ok(id)
    }
    pub fn remove_entity_by_id(&mut self, id: I) -> Result<EntityOwnedById<E>, NoSuchId> {
        let coord = self.coords.remove(&id).ok_or(NoSuchId)?;
        let cell = self
            .grid
            .get_checked_mut(coord)
            .take()
            .expect("cell should contain entity");
        assert_eq!(cell.raw_id, id.to_raw());
        Ok(EntityOwnedById {
            coord,
            entity: cell.entity,
        })
    }
    pub fn get_entity_by_id(&self, id: I) -> Result<EntityById<E>, NoSuchId> {
        let &coord = self.coords.get(&id).ok_or(NoSuchId)?;
        let cell = self
            .grid
            .get_checked(coord)
            .as_ref()
            .expect("cell should contain entity");
        assert_eq!(cell.raw_id, id.to_raw());
        Ok(EntityById {
            coord,
            entity: &cell.entity,
        })
    }
    pub fn get_entity_mut_by_id(&mut self, id: I) -> Result<EntityMutById<E>, NoSuchId> {
        let &coord = self.coords.get(&id).ok_or(NoSuchId)?;
        let cell = self
            .grid
            .get_checked_mut(coord)
            .as_mut()
            .expect("cell should contain entity");
        assert_eq!(cell.raw_id, id.to_raw());
        Ok(EntityMutById {
            coord,
            entity: &mut cell.entity,
        })
    }
    pub fn remove_entity_by_coord(&mut self, coord: Coord) -> Result<Option<EntityOwnedByCoord<I, E>>, OutOfBounds> {
        let cell = self.grid.get_mut(coord).ok_or(OutOfBounds)?;
        Ok(cell.take().map(|Cell { raw_id, entity }| EntityOwnedByCoord {
            id: I::from_raw(raw_id),
            entity,
        }))
    }
    pub fn get_entity_by_coord(&self, coord: Coord) -> Result<Option<EntityByCoord<I, E>>, OutOfBounds> {
        let cell = self.grid.get(coord).ok_or(OutOfBounds)?;
        Ok(cell.as_ref().map(|Cell { raw_id, entity }| EntityByCoord {
            id: I::from_raw(*raw_id),
            entity,
        }))
    }
    pub fn get_entity_mut_by_coord(&mut self, coord: Coord) -> Result<Option<EntityMutByCoord<I, E>>, OutOfBounds> {
        let cell = self.grid.get_mut(coord).ok_or(OutOfBounds)?;
        Ok(cell.as_mut().map(|Cell { raw_id, entity }| EntityMutByCoord {
            id: I::from_raw(*raw_id),
            entity,
        }))
    }
    pub fn get_coord_by_id(&self, id: I) -> Result<Coord, NoSuchId> {
        Ok(*(self.coords.get(&id).ok_or(NoSuchId)?))
    }
    pub fn move_entity_to_empty_cell_by_id(
        &mut self,
        id: I,
        destination_coord: Coord,
    ) -> Result<(), MoveEntityToEmptyCellError<I>> {
        let select = self
            .select_entity_to_move_to_empty_cell(id)
            .map_err(|NoSuchId| MoveEntityToEmptyCellError::NoSuchId)?;
        let staged = select
            .stage_move_entity_to_empty_cell(destination_coord)
            .map_err(|e| match e {
                StageMoveEntityToEmptyCellError::SourceAndDestinationAreEqual => {
                    MoveEntityToEmptyCellError::SourceAndDestinationAreEqual
                }
                StageMoveEntityToEmptyCellError::OutOfBounds => MoveEntityToEmptyCellError::OutOfBounds,
                StageMoveEntityToEmptyCellError::OccupiedBy(id) => MoveEntityToEmptyCellError::OccupiedBy(id),
            })?;
        staged.commit();
        Ok(())
    }
    pub fn select_entity_to_move_to_empty_cell(
        &mut self,
        id: I,
    ) -> Result<SelectEntityToMoveToEmptyCell<I, E>, NoSuchId> {
        let coord = self.coords.get_mut(&id).ok_or(NoSuchId)?;
        Ok(SelectEntityToMoveToEmptyCell {
            coord,
            id,
            grid: &mut self.grid,
        })
    }
    pub fn grid_iter(&self) -> impl Iterator<Item = Option<EntityByCoord<I, E>>> {
        self.grid.iter().map(|maybe_cell| {
            maybe_cell.as_ref().map(|Cell { raw_id, entity }| EntityByCoord {
                id: I::from_raw(*raw_id),
                entity,
            })
        })
    }
    pub fn grid_enumerate(&self) -> impl Iterator<Item = (Coord, Option<EntityByCoord<I, E>>)> {
        self.grid.enumerate().map(|(coord, maybe_cell)| {
            (
                coord,
                maybe_cell.as_ref().map(|Cell { raw_id, entity }| EntityByCoord {
                    id: I::from_raw(*raw_id),
                    entity,
                }),
            )
        })
    }
}

pub struct EntityOwnedById<E> {
    pub coord: Coord,
    pub entity: E,
}

pub struct EntityById<'a, E> {
    pub coord: Coord,
    pub entity: &'a E,
}

pub struct EntityMutById<'a, E> {
    pub coord: Coord,
    pub entity: &'a mut E,
}

pub struct EntityOwnedByCoord<I: IdTrait, E> {
    pub id: I,
    pub entity: E,
}

pub struct EntityByCoord<'a, I: IdTrait, E> {
    pub id: I,
    pub entity: &'a E,
}

pub struct EntityMutByCoord<'a, I: IdTrait, E> {
    pub id: I,
    pub entity: &'a mut E,
}

#[derive(Debug)]
pub enum EmptyCellError<I: IdTrait> {
    OutOfBounds,
    OccupiedBy(I),
}

#[derive(Debug)]
pub struct NoSuchId;

#[derive(Debug)]
pub struct OutOfBounds;

#[derive(Debug)]
pub enum StageMoveEntityToEmptyCellError<I: IdTrait> {
    SourceAndDestinationAreEqual,
    OutOfBounds,
    OccupiedBy(I),
}

#[derive(Debug)]
pub enum MoveEntityToEmptyCellError<I: IdTrait> {
    NoSuchId,
    SourceAndDestinationAreEqual,
    OutOfBounds,
    OccupiedBy(I),
}

pub struct SelectEntityToMoveToEmptyCell<'a, I: IdTrait, E> {
    id: I,
    coord: &'a mut Coord,
    grid: &'a mut Grid<Option<Cell<E>>>,
}

pub struct StageMoveEntityToEmptyCell<'a, E> {
    source_coord: &'a mut Coord,
    destination_coord: Coord,
    source_cell: &'a mut Option<Cell<E>>,
    destination_cell: &'a mut Option<Cell<E>>,
}

impl<'a, I: IdTrait, E> SelectEntityToMoveToEmptyCell<'a, I, E> {
    pub fn coord(&self) -> Coord {
        *self.coord
    }
    pub fn stage_move_entity_to_empty_cell(
        self,
        destination_coord: Coord,
    ) -> Result<StageMoveEntityToEmptyCell<'a, E>, StageMoveEntityToEmptyCellError<I>> {
        match self.grid.get2_mut(*self.coord, destination_coord) {
            Err(Get2Error::CoordsEqual) => Err(StageMoveEntityToEmptyCellError::SourceAndDestinationAreEqual),
            Err(Get2Error::LeftOutOfBounds) => panic!("source coord should always be in bound"),
            Err(Get2Error::RightOutOfBounds) => Err(StageMoveEntityToEmptyCellError::OutOfBounds),
            Ok((source_cell, destination_cell)) => {
                assert_eq!(
                    source_cell
                        .as_ref()
                        .expect("source cell should always contain entity")
                        .raw_id,
                    self.id.to_raw()
                );
                match destination_cell.as_ref() {
                    Some(Cell { raw_id, .. }) => Err(StageMoveEntityToEmptyCellError::OccupiedBy(I::from_raw(*raw_id))),
                    None => Ok(StageMoveEntityToEmptyCell {
                        source_coord: self.coord,
                        destination_coord,
                        source_cell,
                        destination_cell,
                    }),
                }
            }
        }
    }
}

impl<'a, E> StageMoveEntityToEmptyCell<'a, E> {
    pub fn entity(&self) -> &E {
        let cell = self.source_cell.as_ref().unwrap();
        &cell.entity
    }
    pub fn commit(self) {
        assert!(self.destination_cell.is_none(), "destination already contains value");
        assert!(self.source_cell.is_some(), "source doesn't contain value");
        mem::swap(self.source_cell, self.destination_cell);
        *self.source_coord = self.destination_coord;
    }
}
