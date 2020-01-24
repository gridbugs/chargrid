use crate::world::{
    data::{Components, Layer, Location},
    query,
    realtime_periodic::data::RealtimeComponents,
    spatial_grid::{LocationUpdate, OccupiedBy, SpatialGrid},
    spawn,
};
use direction::CardinalDirection;
use ecs::{Ecs, Entity};
use grid_2d::Coord;

pub fn character_walk_in_direction(
    ecs: &mut Ecs<Components>,
    spatial_grid: &mut SpatialGrid,
    entity: Entity,
    direction: CardinalDirection,
) {
    let current_location = ecs.components.location.get_mut(entity).unwrap();
    debug_assert_eq!(current_location.layer, Layer::Character);
    let target_coord = current_location.coord + direction.coord();
    if query::is_solid_feature_at_coord(&ecs, spatial_grid, target_coord) {
        return;
    }
    let target_location = Location {
        coord: target_coord,
        layer: Layer::Character,
    };
    if let Err(OccupiedBy(_occupant)) = spatial_grid.location_update(ecs, entity, target_location) {
        // TODO melee
    }
}

pub fn character_fire_rocket(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    character: Entity,
    target: Coord,
) {
    let character_coord = ecs.components.location.get(character).unwrap().coord;
    if character_coord == target {
        return;
    }
    spawn::rocket(ecs, realtime_components, spatial_grid, character_coord, target);
}
