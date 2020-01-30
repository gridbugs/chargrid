use crate::world::{
    data::{Components, Layer, Location, OnCollision, ProjectileDamage},
    query,
    realtime_periodic::{core::ScheduledRealtimePeriodicState, data::RealtimeComponents, movement},
    spatial_grid::{LocationUpdate, OccupiedBy, SpatialGrid},
    spawn, ExternalEvent,
};
use direction::{CardinalDirection, Direction};
use ecs::{ComponentsTrait, Ecs, Entity};
use grid_2d::Coord;
use line_2d::{LineSegment, StartAndEndAreTheSame};
use std::time::Duration;

pub fn character_walk_in_direction(
    ecs: &mut Ecs<Components>,
    spatial_grid: &mut SpatialGrid,
    entity: Entity,
    direction: CardinalDirection,
) {
    let &current_location = ecs.components.location.get(entity).unwrap();
    debug_assert_eq!(current_location.layer, Some(Layer::Character));
    let target_coord = current_location.coord + direction.coord();
    if query::is_solid_feature_at_coord(&ecs, spatial_grid, target_coord) {
        return;
    }
    let target_location = Location {
        coord: target_coord,
        ..current_location
    };
    if let Err(OccupiedBy(_occupant)) = spatial_grid.update_entity_location(ecs, entity, target_location) {
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
        debug_assert_eq!(current_location.layer, Some(Layer::Character));
        let target_coord = current_location.coord + direction.coord();
        if query::is_solid_feature_at_coord(&ecs, spatial_grid, target_coord) {
            return;
        }
        let target_location = Location {
            coord: target_coord,
            ..current_location
        };
        let _ignore_if_occupied = spatial_grid.update_entity_location(ecs, entity, target_location);
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

enum ExplosionHit {
    Direct,
    Indirect(LineSegment),
}

fn does_explosion_hit_entity(
    ecs: &Ecs<Components>,
    spatial_grid: &SpatialGrid,
    explosion_coord: Coord,
    explosion_range: u32,
    entity: Entity,
) -> Option<ExplosionHit> {
    if let Some(Location {
        coord: entity_coord, ..
    }) = ecs.components.location.get(entity)
    {
        match LineSegment::try_new(explosion_coord, *entity_coord) {
            Ok(explosion_to_entity) => {
                for (i, coord) in explosion_to_entity.iter().enumerate() {
                    if i > explosion_range as usize {
                        return None;
                    }
                    let spatial_cell = spatial_grid.get_checked(coord);
                    if let Some(feature_entity) = spatial_cell.feature {
                        if ecs.components.solid.contains(feature_entity) {
                            return None;
                        }
                    }
                }
                Some(ExplosionHit::Indirect(explosion_to_entity))
            }
            Err(StartAndEndAreTheSame) => Some(ExplosionHit::Direct),
        }
    } else {
        None
    }
}

fn explosion(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    coord: Coord,
    range: u32,
    external_events: &mut Vec<ExternalEvent>,
) {
    spawn::explosion(ecs, realtime_components, spatial_grid, coord, external_events);
    for character_entity in ecs.components.character.entities() {
        if let Some(explosion_hit) = does_explosion_hit_entity(ecs, spatial_grid, coord, range, character_entity) {
            let character_coord = ecs.components.location.get(character_entity).unwrap().coord;
            let character_to_explosion_distance_squared = coord.distance2(character_coord);
            match explosion_hit {
                ExplosionHit::Direct => {
                    let mut solid_neighbour_vector = Coord::new(0, 0);
                    for direction in CardinalDirection::all() {
                        let neighbour_coord = coord + direction.coord();
                        if let Some(spatial_cell) = spatial_grid.get(neighbour_coord) {
                            if spatial_cell.feature.is_some() || spatial_cell.character.is_some() {
                                solid_neighbour_vector += direction.coord();
                            }
                        }
                    }
                    if !solid_neighbour_vector.is_zero() {
                        let travel_vector = -solid_neighbour_vector;
                        ecs.components.realtime.insert(character_entity, ());
                        realtime_components.movement.insert(
                            character_entity,
                            ScheduledRealtimePeriodicState {
                                state: movement::spec::Movement {
                                    path: travel_vector,
                                    repeat: movement::spec::Repeat::N((range / 3) as usize),
                                    cardinal_step_duration: Duration::from_millis(100),
                                }
                                .build(),
                                until_next_event: Duration::from_millis(0),
                            },
                        );
                    }
                }
                ExplosionHit::Indirect(path) => {
                    let push_back = 1 + (range / (2 * (character_to_explosion_distance_squared + 1)));
                    ecs.components.realtime.insert(character_entity, ());
                    realtime_components.movement.insert(
                        character_entity,
                        ScheduledRealtimePeriodicState {
                            state: movement::spec::Movement {
                                path: path.delta(),
                                repeat: movement::spec::Repeat::N(push_back as usize),
                                cardinal_step_duration: Duration::from_millis(100),
                            }
                            .build(),
                            until_next_event: Duration::from_millis(0),
                        },
                    );
                }
            }
        }
    }
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
                    explosion(
                        ecs,
                        realtime_components,
                        spatial_grid,
                        current_coord,
                        10, // range
                        external_events,
                    );
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
        let _ = spatial_grid.update_entity_location(
            ecs,
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
