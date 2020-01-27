use crate::world::{
    data::{Components, Layer, Location, OnCollision, ProjectileDamage},
    query,
    realtime_periodic::data::RealtimeComponents,
    spatial_grid::{LocationUpdate, OccupiedBy, SpatialGrid},
    spawn, ExternalEvent,
};
use direction::{CardinalDirection, Direction};
use ecs::{ComponentsTrait, Ecs, Entity};
use grid_2d::Coord;

pub fn character_walk_in_direction(
    ecs: &mut Ecs<Components>,
    spatial_grid: &mut SpatialGrid,
    entity: Entity,
    direction: CardinalDirection,
) {
    let &current_location = ecs.components.location.get(entity).unwrap();
    debug_assert_eq!(current_location.layer, Layer::Character);
    let target_coord = current_location.coord + direction.coord();
    if query::is_solid_feature_at_coord(&ecs, spatial_grid, target_coord) {
        return;
    }
    let target_location = Location {
        coord: target_coord,
        ..current_location
    };
    if let Err(OccupiedBy(_occupant)) = spatial_grid.location_update(ecs, entity, target_location) {
        // TODO melee
    }
}

fn character_push_in_direction(
    ecs: &mut Ecs<Components>,
    spatial_grid: &mut SpatialGrid,
    entity: Entity,
    direction: Direction,
) {
    if let Some(&current_location) = ecs.components.location.get(entity) {
        debug_assert_eq!(current_location.layer, Layer::Character);
        let target_coord = current_location.coord + direction.coord();
        if query::is_solid_feature_at_coord(&ecs, spatial_grid, target_coord) {
            return;
        }
        let target_location = Location {
            coord: target_coord,
            ..current_location
        };
        let _ignore_if_occupied = spatial_grid.location_update(ecs, entity, target_location);
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

fn die(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, entity: Entity) {
    spatial_grid.remove_entity(ecs, entity);
}

fn add_blood_stain_to_floor(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) {
    if let Some(floor_entity) = spatial_grid.get_checked(coord).floor {
        ecs.components.blood.insert(floor_entity, ());
    }
}

fn take_damage(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, entity: Entity, hit_points_to_lose: u32) {
    if let Some(hit_points) = ecs.components.hit_points.get_mut(entity) {
        let coord = ecs.components.location.get(entity).unwrap().coord;
        if let Some(remaining_hit_points) = hit_points.current.checked_sub(hit_points_to_lose) {
            hit_points.current = remaining_hit_points;
            if ecs.components.player.contains(entity) {
                println!("{:?}", hit_points);
            }
        } else {
            die(ecs, spatial_grid, entity);
        }
        add_blood_stain_to_floor(ecs, spatial_grid, coord);
    } else {
        log::warn!("attempt to damage entity without hit_points component");
    }
}

fn apply_projectile_damage(
    ecs: &mut Ecs<Components>,
    spatial_grid: &mut SpatialGrid,
    projectile_entity: Entity,
    projectile_damage: ProjectileDamage,
    projectile_movement_direction: Direction,
    entity_to_damage: Entity,
) {
    take_damage(ecs, spatial_grid, entity_to_damage, projectile_damage.hit_points);
    if projectile_damage.push_back {
        character_push_in_direction(ecs, spatial_grid, entity_to_damage, projectile_movement_direction);
    }
    ecs.remove(projectile_entity);
}

pub fn projectile_stop(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    projectile_entity: Entity,
    external_events: &mut Vec<ExternalEvent>,
) {
    if let Some(&current_location) = ecs.components.location.get(projectile_entity) {
        if let Some(on_collision) = ecs.components.on_collision.get(projectile_entity) {
            let current_coord = current_location.coord;
            match on_collision {
                OnCollision::Explode => {
                    spawn::explosion(ecs, realtime_components, spatial_grid, current_coord, external_events);
                    ecs.remove(projectile_entity);
                    realtime_components.remove_entity(projectile_entity);
                }
            }
        }
    }
    realtime_components.movement.remove(projectile_entity);
}

pub fn projectile_move(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    projectile_entity: Entity,
    movement_direction: Direction,
    external_events: &mut Vec<ExternalEvent>,
) {
    if let Some(&current_location) = ecs.components.location.get(projectile_entity) {
        debug_assert!(current_location.is_untracked());
        let next_coord = current_location.coord + movement_direction.coord();
        let colides_with = ecs
            .components
            .colides_with
            .get(projectile_entity)
            .cloned()
            .unwrap_or_default();
        let &spatial_cell = spatial_grid.get_checked(next_coord);
        if let Some(character_entity) = spatial_cell.character {
            if let Some(&projectile_damage) = ecs.components.projectile_damage.get(projectile_entity) {
                apply_projectile_damage(
                    ecs,
                    spatial_grid,
                    projectile_entity,
                    projectile_damage,
                    movement_direction,
                    character_entity,
                );
            }
        }
        if let Some(entity_in_cell) = spatial_cell.feature.or(spatial_cell.character) {
            if (colides_with.solid && ecs.components.solid.contains(entity_in_cell))
                || (colides_with.character && ecs.components.character.contains(entity_in_cell))
            {
                projectile_stop(
                    ecs,
                    realtime_components,
                    spatial_grid,
                    projectile_entity,
                    external_events,
                );
                return;
            }
        }
        ecs.components.location.insert(
            projectile_entity,
            Location {
                coord: next_coord,
                ..current_location
            },
        );
    } else {
        ecs.remove(projectile_entity);
        realtime_components.remove_entity(projectile_entity);
    }
}
