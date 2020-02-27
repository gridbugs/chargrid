use crate::world::{
    realtime_periodic::{core::ScheduledRealtimePeriodicState, movement},
    ExternalEvent, World,
};
use direction::Direction;
use ecs::Entity;
use grid_2d::Coord;
use line_2d::LineSegment;
use std::time::Duration;

pub mod spec {
    pub use grid_2d::Coord;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct ParticleEmitter {
        pub duration: Duration,
        pub num_particles_per_frame: u32,
        pub min_step: Duration,
        pub max_step: Duration,
        pub fade_duration: Duration,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Mechanics {
        pub range: u32,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Explosion {
        pub mechanics: Mechanics,
        pub particle_emitter: ParticleEmitter,
    }
}

struct CharacterEffect {
    push_back: u32,
    damage: u32,
}

fn character_effect_indirect_hit(mechanics: &spec::Mechanics, explosion_to_character: LineSegment) -> CharacterEffect {
    let character_to_explosion_distance_squared = explosion_to_character.delta().magnitude2();
    let push_back = 1 + (mechanics.range / (2 * (character_to_explosion_distance_squared + 1)));
    CharacterEffect {
        push_back,
        damage: push_back * 10,
    }
}

fn apply_indirect_hit(
    world: &mut World,
    mechanics: &spec::Mechanics,
    character_entity: Entity,
    explosion_to_character: LineSegment,
) {
    let CharacterEffect { push_back, damage } = character_effect_indirect_hit(mechanics, explosion_to_character);
    world.components.realtime.insert(character_entity, ());
    world.realtime_components.movement.insert(
        character_entity,
        ScheduledRealtimePeriodicState {
            state: movement::spec::Movement {
                path: explosion_to_character.delta(),
                repeat: movement::spec::Repeat::Steps(push_back as usize),
                cardinal_step_duration: Duration::from_millis(100),
            }
            .build(),
            until_next_event: Duration::from_millis(0),
        },
    );
    world.damage_character(character_entity, damage);
}

fn character_effect_direct_hit(mechanics: &spec::Mechanics) -> CharacterEffect {
    let push_back = mechanics.range / 3;
    CharacterEffect {
        push_back,
        damage: mechanics.range * 10,
    }
}

fn apply_direct_hit(world: &mut World, explosion_coord: Coord, mechanics: &spec::Mechanics, character_entity: Entity) {
    let mut solid_neighbour_vector = Coord::new(0, 0);
    for direction in Direction::all() {
        let neighbour_coord = explosion_coord + direction.coord();
        if let Some(spatial_cell) = world.spatial.get_cell(neighbour_coord) {
            if spatial_cell.feature.is_some() || spatial_cell.character.is_some() {
                solid_neighbour_vector += direction.coord();
            }
        }
    }
    let CharacterEffect { push_back, damage } = character_effect_direct_hit(mechanics);
    if solid_neighbour_vector.is_zero() {
        log::warn!("Direct hit with no solid neighbours shouldn't be possible.");
    } else {
        let travel_vector = -solid_neighbour_vector;
        world.components.realtime.insert(character_entity, ());
        world.realtime_components.movement.insert(
            character_entity,
            ScheduledRealtimePeriodicState {
                state: movement::spec::Movement {
                    path: travel_vector,
                    repeat: movement::spec::Repeat::Steps(push_back as usize),
                    cardinal_step_duration: Duration::from_millis(100),
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
    }
    world.damage_character(character_entity, damage);
}

fn is_in_explosion_range(explosion_coord: Coord, mechanics: &spec::Mechanics, coord: Coord) -> bool {
    explosion_coord.distance2(coord) <= mechanics.range.pow(2)
}

fn apply_mechanics(world: &mut World, explosion_coord: Coord, mechanics: &spec::Mechanics) {
    for character_entity in world.components.character.entities().collect::<Vec<_>>() {
        if let Some(&character_coord) = world.spatial.coord(character_entity) {
            if character_coord == explosion_coord {
                apply_direct_hit(world, explosion_coord, mechanics, character_entity);
            } else {
                if !is_in_explosion_range(explosion_coord, mechanics, character_coord) {
                    continue;
                }
                let explosion_to_character = LineSegment::new(explosion_coord, character_coord);
                if !world.is_solid_feature_in_line_segment(explosion_to_character) {
                    apply_indirect_hit(world, mechanics, character_entity, explosion_to_character);
                } else {
                    continue;
                }
            }
        }
    }
}

pub fn explode(world: &mut World, coord: Coord, explosion: spec::Explosion, external_events: &mut Vec<ExternalEvent>) {
    world.spawn_explosion_emitter(coord, &explosion.particle_emitter);
    apply_mechanics(world, coord, &explosion.mechanics);
    external_events.push(ExternalEvent::Explosion(coord));
}
