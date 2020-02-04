use crate::world::{
    data::{Layer, Location, OnCollision, ProjectileDamage},
    explosion,
    spatial_grid::{LocationUpdate, OccupiedBy},
    spawn::WorldSpawn,
    ExternalEvent, World, WorldQuery,
};
use direction::{CardinalDirection, Direction};
use ecs::{ComponentsTrait, Entity};
use grid_2d::Coord;

pub trait WorldAction {
    fn character_walk_in_direction(&mut self, character: Entity, direction: CardinalDirection);
    fn character_fire_rocket(&mut self, character: Entity, target: Coord);
    fn projectile_stop(&mut self, projectile_entity: Entity, external_events: &mut Vec<ExternalEvent>);
    fn projectile_move(
        &mut self,
        projectile_entity: Entity,
        movement_direction: Direction,
        external_events: &mut Vec<ExternalEvent>,
    );
    fn damage_character(&mut self, character: Entity, hit_points_to_lose: u32);
}

impl WorldAction for World {
    fn character_walk_in_direction(&mut self, character: Entity, direction: CardinalDirection) {
        let &current_location = self.ecs.components.location.get(character).unwrap();
        debug_assert_eq!(current_location.layer, Some(Layer::Character));
        let target_coord = current_location.coord + direction.coord();
        if self.is_solid_feature_at_coord(target_coord) {
            return;
        }
        let target_location = Location {
            coord: target_coord,
            ..current_location
        };
        if let Err(OccupiedBy(_occupant)) =
            self.spatial_grid
                .update_entity_location(&mut self.ecs, character, target_location)
        {
            // TODO melee
        }
    }

    fn character_fire_rocket(&mut self, character: Entity, target: Coord) {
        let character_coord = self.ecs.components.location.get(character).unwrap().coord;
        if character_coord == target {
            return;
        }
        self.spawn_rocket(character_coord, target);
    }

    fn projectile_stop(&mut self, projectile_entity: Entity, external_events: &mut Vec<ExternalEvent>) {
        if let Some(&current_location) = self.ecs.components.location.get(projectile_entity) {
            if let Some(on_collision) = self.ecs.components.on_collision.get(projectile_entity).cloned() {
                let current_coord = current_location.coord;
                match on_collision {
                    OnCollision::Explode(explosion_spec) => {
                        explosion::explode(self, current_coord, explosion_spec, external_events);
                        self.ecs.remove(projectile_entity);
                        self.realtime_components.remove_entity(projectile_entity);
                    }
                }
            }
        }
        self.realtime_components.movement.remove(projectile_entity);
    }

    fn projectile_move(
        &mut self,
        projectile_entity: Entity,
        movement_direction: Direction,
        external_events: &mut Vec<ExternalEvent>,
    ) {
        if let Some(&current_location) = self.ecs.components.location.get(projectile_entity) {
            let next_coord = current_location.coord + movement_direction.coord();
            let colides_with = self
                .ecs
                .components
                .colides_with
                .get(projectile_entity)
                .cloned()
                .unwrap_or_default();
            let &spatial_cell = self.spatial_grid.get_checked(next_coord);
            if let Some(character_entity) = spatial_cell.character {
                if let Some(&projectile_damage) = self.ecs.components.projectile_damage.get(projectile_entity) {
                    self.apply_projectile_damage(
                        projectile_entity,
                        projectile_damage,
                        movement_direction,
                        character_entity,
                    );
                }
            }
            if let Some(entity_in_cell) = spatial_cell.feature.or(spatial_cell.character) {
                if (colides_with.solid && self.ecs.components.solid.contains(entity_in_cell))
                    || (colides_with.character && self.ecs.components.character.contains(entity_in_cell))
                {
                    self.projectile_stop(projectile_entity, external_events);
                    return;
                }
            }
            let _ = self.spatial_grid.update_entity_location(
                &mut self.ecs,
                projectile_entity,
                Location {
                    coord: next_coord,
                    ..current_location
                },
            );
        } else {
            self.ecs.remove(projectile_entity);
            self.realtime_components.remove_entity(projectile_entity);
        }
    }

    fn damage_character(&mut self, character: Entity, hit_points_to_lose: u32) {
        if let Some(hit_points) = self.ecs.components.hit_points.get_mut(character) {
            let coord = self.ecs.components.location.get(character).unwrap().coord;
            match hit_points.current.checked_sub(hit_points_to_lose) {
                None | Some(0) => {
                    hit_points.current = 0;
                    self.character_die(character);
                }
                Some(non_zero_remaining_hit_points) => {
                    hit_points.current = non_zero_remaining_hit_points;
                }
            }
            self.add_blood_stain_to_floor(coord);
        } else {
            log::warn!("attempt to damage entity without hit_points component");
        }
    }
}

trait WorldActionPrivate {
    fn character_push_in_direction(&mut self, entity: Entity, direction: Direction);
    fn character_die(&mut self, character: Entity);
    fn add_blood_stain_to_floor(&mut self, coord: Coord);
    fn apply_projectile_damage(
        &mut self,
        projectile_entity: Entity,
        projectile_damage: ProjectileDamage,
        projectile_movement_direction: Direction,
        entity_to_damage: Entity,
    );
}

impl WorldActionPrivate for World {
    fn character_push_in_direction(&mut self, entity: Entity, direction: Direction) {
        if let Some(&current_location) = self.ecs.components.location.get(entity) {
            debug_assert_eq!(current_location.layer, Some(Layer::Character));
            let target_coord = current_location.coord + direction.coord();
            if self.is_solid_feature_at_coord(target_coord) {
                return;
            }
            let target_location = Location {
                coord: target_coord,
                ..current_location
            };
            let _ignore_if_occupied = self
                .spatial_grid
                .update_entity_location(&mut self.ecs, entity, target_location);
        }
    }

    fn character_die(&mut self, character: Entity) {
        self.spatial_grid.remove_entity(&mut self.ecs, character);
    }

    fn add_blood_stain_to_floor(&mut self, coord: Coord) {
        if let Some(floor_entity) = self.spatial_grid.get_checked(coord).floor {
            self.ecs.components.blood.insert(floor_entity, ());
        }
    }

    fn apply_projectile_damage(
        &mut self,
        projectile_entity: Entity,
        projectile_damage: ProjectileDamage,
        projectile_movement_direction: Direction,
        entity_to_damage: Entity,
    ) {
        self.damage_character(entity_to_damage, projectile_damage.hit_points);
        if projectile_damage.push_back {
            self.character_push_in_direction(entity_to_damage, projectile_movement_direction);
        }
        self.ecs.remove(projectile_entity);
    }
}
