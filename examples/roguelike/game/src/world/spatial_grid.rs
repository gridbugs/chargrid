use crate::world::data::Components;
use ecs::{ComponentTable, Ecs, Entity};
use grid_2d::{Coord, Grid};
use serde::{Deserialize, Serialize};

pub type SpatialGrid = Grid<SpatialCell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Floor,
    Feature,
    Character,
    Particle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub coord: Coord,
    pub layer: Layer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpatialCell {
    pub floor: Option<Entity>,
    pub feature: Option<Entity>,
    pub character: Option<Entity>,
}

impl Default for SpatialCell {
    fn default() -> Self {
        Self {
            floor: None,
            feature: None,
            character: None,
        }
    }
}

enum SelectFieldMut<'a> {
    Tracked(&'a mut Option<Entity>),
    Untracked,
}

#[derive(Debug)]
pub struct OccupiedBy(pub Entity);

impl SpatialCell {
    fn select_field_mut(&mut self, layer: Layer) -> SelectFieldMut {
        match layer {
            Layer::Character => SelectFieldMut::Tracked(&mut self.character),
            Layer::Feature => SelectFieldMut::Tracked(&mut self.feature),
            Layer::Floor => SelectFieldMut::Tracked(&mut self.floor),
            Layer::Particle => SelectFieldMut::Untracked,
        }
    }
    fn insert(&mut self, entity: Entity, layer: Layer) -> Result<(), OccupiedBy> {
        let layer_field = match self.select_field_mut(layer) {
            SelectFieldMut::Tracked(layer_field) => layer_field,
            SelectFieldMut::Untracked => return Ok(()),
        };
        if let Some(&occupant) = layer_field.as_ref() {
            Err(OccupiedBy(occupant))
        } else {
            *layer_field = Some(entity);
            Ok(())
        }
    }
    fn clear(&mut self, layer: Layer) -> Option<Entity> {
        match self.select_field_mut(layer) {
            SelectFieldMut::Tracked(field) => field.take(),
            SelectFieldMut::Untracked => None,
        }
    }
}

pub trait LocationUpdate {
    fn component_location_update(
        &mut self,
        location_component: &mut ComponentTable<Location>,
        entity: Entity,
        location: Location,
    ) -> Result<(), OccupiedBy>;
    fn location_update(
        &mut self,
        ecs: &mut Ecs<Components>,
        entity: Entity,
        location: Location,
    ) -> Result<(), OccupiedBy> {
        self.component_location_update(&mut ecs.components.location, entity, location)
    }
}

impl LocationUpdate for SpatialGrid {
    fn component_location_update(
        &mut self,
        location_component: &mut ComponentTable<Location>,
        entity: Entity,
        location: Location,
    ) -> Result<(), OccupiedBy> {
        let cell = self.get_checked_mut(location.coord);
        cell.insert(entity, location.layer)?;
        if let Some(original_location) = location_component.insert(entity, location) {
            let original_cell = self.get_checked_mut(original_location.coord);
            let should_match_entity = original_cell.clear(original_location.layer);
            debug_assert_eq!(
                should_match_entity,
                Some(entity),
                "Current location of entity doesn't contain entity in spatial grid"
            );
        }
        Ok(())
    }
}
