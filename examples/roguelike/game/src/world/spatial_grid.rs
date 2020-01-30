use crate::world::data::Components;
use ecs::{Ecs, Entity};
use grid_2d::{Coord, Grid};
use serde::{Deserialize, Serialize};

pub type SpatialGrid = Grid<SpatialCell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Floor,
    Feature,
    Character,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub coord: Coord,
    pub layer: Option<Layer>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct OccupiedBy(pub Entity);

impl SpatialCell {
    fn select_field_mut(&mut self, layer: Layer) -> &mut Option<Entity> {
        match layer {
            Layer::Character => &mut self.character,
            Layer::Feature => &mut self.feature,
            Layer::Floor => &mut self.floor,
        }
    }
    fn insert(&mut self, entity: Entity, layer: Layer) -> Result<(), OccupiedBy> {
        let layer_field = self.select_field_mut(layer);
        if let Some(&occupant) = layer_field.as_ref() {
            Err(OccupiedBy(occupant))
        } else {
            *layer_field = Some(entity);
            Ok(())
        }
    }
    fn clear(&mut self, layer: Layer) -> Option<Entity> {
        self.select_field_mut(layer).take()
    }
}

pub trait LocationUpdate {
    fn update_entity_location(
        &mut self,
        ecs: &mut Ecs<Components>,
        entity: Entity,
        location: Location,
    ) -> Result<(), OccupiedBy>;
    fn remove_entity(&mut self, ecs: &mut Ecs<Components>, entity: Entity);
}

impl LocationUpdate for SpatialGrid {
    fn update_entity_location(
        &mut self,
        ecs: &mut Ecs<Components>,
        entity: Entity,
        location: Location,
    ) -> Result<(), OccupiedBy> {
        if let Some(layer) = location.layer {
            let cell = self.get_checked_mut(location.coord);
            cell.insert(entity, layer)?;
        }
        if let Some(original_location) = ecs.components.location.insert(entity, location) {
            let original_cell = self.get_checked_mut(original_location.coord);
            if let Some(original_layer) = original_location.layer {
                let should_match_entity = original_cell.clear(original_layer);
                debug_assert_eq!(
                    should_match_entity,
                    Some(entity),
                    "Current location of entity doesn't contain entity in spatial grid"
                );
            }
        }
        Ok(())
    }
    fn remove_entity(&mut self, ecs: &mut Ecs<Components>, entity: Entity) {
        if let Some(location) = ecs.components.location.get(entity) {
            if let Some(layer) = location.layer {
                if let Some(cell) = self.get_mut(location.coord) {
                    cell.clear(layer);
                }
            }
        }
        ecs.remove(entity);
    }
}
