use crate::world::{
    action,
    data::Components,
    query,
    realtime_periodic::{core::ScheduledRealtimePeriodicState, data::RealtimeComponents, movement},
    spatial_grid::{Location, SpatialGrid},
    spawn, ExternalEvent,
};
use direction::CardinalDirection;
use ecs::{ComponentTable, Ecs, Entity};
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

struct CharacterDamage(u32);

fn character_effect_indirect_hit(mechanics: &spec::Mechanics, explosion_to_character: LineSegment) -> CharacterEffect {
    let character_to_explosion_distance_squared = explosion_to_character.delta().magnitude2();
    let push_back = 1 + (mechanics.range / (2 * (character_to_explosion_distance_squared + 1)));
    CharacterEffect {
        push_back,
        damage: push_back,
    }
}

fn apply_indirect_hit(
    realtime_component_table: &mut ComponentTable<()>,
    realtime_components: &mut RealtimeComponents,
    mechanics: &spec::Mechanics,
    character_entity: Entity,
    explosion_to_character: LineSegment,
) -> CharacterDamage {
    let CharacterEffect { push_back, damage } = character_effect_indirect_hit(mechanics, explosion_to_character);
    realtime_component_table.insert(character_entity, ());
    realtime_components.movement.insert(
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
    CharacterDamage(damage)
}

fn character_effect_direct_hit(mechanics: &spec::Mechanics) -> CharacterEffect {
    let push_back = mechanics.range / 3;
    CharacterEffect {
        push_back,
        damage: mechanics.range,
    }
}

fn apply_direct_hit(
    realtime_component_table: &mut ComponentTable<()>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    explosion_coord: Coord,
    mechanics: &spec::Mechanics,
    character_entity: Entity,
) -> CharacterDamage {
    let mut solid_neighbour_vector = Coord::new(0, 0);
    for direction in CardinalDirection::all() {
        let neighbour_coord = explosion_coord + direction.coord();
        if let Some(spatial_cell) = spatial_grid.get(neighbour_coord) {
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
        realtime_component_table.insert(character_entity, ());
        realtime_components.movement.insert(
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
    CharacterDamage(damage)
}

fn is_in_explosion_range(explosion_coord: Coord, mechanics: &spec::Mechanics, coord: Coord) -> bool {
    explosion_coord.distance2(coord) <= mechanics.range.pow(2)
}

fn apply_mechanics(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    explosion_coord: Coord,
    mechanics: &spec::Mechanics,
) {
    for character_entity in ecs.components.character.entities().collect::<Vec<_>>() {
        if let Some(&Location {
            coord: character_coord, ..
        }) = ecs.components.location.get(character_entity)
        {
            let CharacterDamage(damage) = if character_coord == explosion_coord {
                apply_direct_hit(
                    &mut ecs.components.realtime,
                    realtime_components,
                    spatial_grid,
                    explosion_coord,
                    mechanics,
                    character_entity,
                )
            } else {
                if !is_in_explosion_range(explosion_coord, mechanics, character_coord) {
                    continue;
                }
                let explosion_to_character = LineSegment::new(explosion_coord, character_coord);
                if !query::is_solid_feature_in_line_segment(ecs, spatial_grid, explosion_to_character) {
                    apply_indirect_hit(
                        &mut ecs.components.realtime,
                        realtime_components,
                        mechanics,
                        character_entity,
                        explosion_to_character,
                    )
                } else {
                    continue;
                }
            };
            action::take_damage(ecs, spatial_grid, character_entity, damage);
        }
    }
}

pub fn explode(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    coord: Coord,
    explosion: spec::Explosion,
    external_events: &mut Vec<ExternalEvent>,
) {
    spawn::explosion_emitter(
        ecs,
        realtime_components,
        spatial_grid,
        coord,
        &explosion.particle_emitter,
    );
    apply_mechanics(ecs, realtime_components, spatial_grid, coord, &explosion.mechanics);
    external_events.push(ExternalEvent::Explosion(coord));
}
