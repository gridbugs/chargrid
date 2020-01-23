use crate::particle::{
    DurationRange, ParticleAngleRange, ParticleEmitterState, ParticleFadeSpec, ParticleInitialFadeProgress,
    ParticleMovementSpec,
};
use crate::rational::Rational;
use crate::realtime_periodic_core::{ScheduledRealtimePeriodicState, TimeConsumingEvent};
use crate::realtime_periodic_data::{MovementState, RealtimeComponents, FRAME_DURATION};
use crate::visibility::Light;
use crate::world_data::{
    is_solid_feature_at_coord, location_insert, Components, Disposition, Layer, Location, Npc, OccupiedBy, OnCollision,
    SpatialCell, Tile,
};
use crate::ExternalEvent;
use direction::{CardinalDirection, Direction};
pub use ecs::Entity;
use ecs::{ecs_components, ComponentTable, Ecs};
use grid_2d::{Coord, Grid, Size};
use line_2d::InfiniteStepIter;
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::vision_distance::Circle;
use std::time::Duration;
use vector::Radial;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    ecs: Ecs<Components>,
    realtime_components: RealtimeComponents,
    spatial_grid: Grid<SpatialCell>,
    realtime_entities: Vec<Entity>,
}

impl World {
    pub fn new(size: Size) -> Self {
        let ecs = Ecs::new();
        let realtime_components = RealtimeComponents::default();
        let spatial_grid = Grid::new_default(size);
        Self {
            ecs,
            realtime_components,
            spatial_grid,
            realtime_entities: Vec::new(),
        }
    }
    pub fn spawn_player(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Player);
        self.ecs.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new(255, 187, 127),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        entity
    }
    pub fn spawn_former_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::FormerHuman);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Hostile,
            },
        );
        entity
    }
    pub fn spawn_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Human);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Afraid,
            },
        );
        entity
    }

    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Floor),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Floor);
        entity
    }
    pub fn spawn_carpet(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Floor),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Carpet);
        entity
    }
    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Feature),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Wall);
        self.ecs.components.solid.insert(entity, ());
        self.ecs.components.opacity.insert(entity, 255);
        entity
    }
    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Feature),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(420),
                diminish: Rational {
                    numerator: 1,
                    denominator: 25,
                },
            },
        );
        entity
    }
    pub fn character_walk_in_direction(&mut self, entity: Entity, direction: CardinalDirection) {
        let current_location = self.ecs.components.location.get_mut(entity).unwrap();
        debug_assert_eq!(current_location.layer, Layer::Character);
        let target_coord = current_location.coord + direction.coord();
        if is_solid_feature_at_coord(target_coord, &self.ecs.components.solid, &self.spatial_grid) {
            return;
        }
        let target_location = Location::new(target_coord, Layer::Character);
        if let Err(OccupiedBy(_occupant)) = location_insert(
            entity,
            target_location,
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        ) {
            // TODO melee
        }
    }
    pub fn character_fire_bullet(&mut self, character: Entity, target: Coord) {
        let character_coord = self.ecs.components.location.get(character).unwrap().coord;
        if character_coord == target {
            return;
        }
        let bullet_entity = self.ecs.create();
        location_insert(
            bullet_entity,
            Location::new(character_coord, Layer::Particle),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.realtime.insert(bullet_entity, ());
        self.ecs.components.blocks_gameplay.insert(bullet_entity, ());
        self.realtime_components.movement.insert(
            bullet_entity,
            ScheduledRealtimePeriodicState {
                state: MovementState::new(
                    InfiniteStepIter::new(target - character_coord),
                    Duration::from_millis(16),
                ),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.realtime_components.particle_emitter.insert(
            bullet_entity,
            ScheduledRealtimePeriodicState {
                state: ParticleEmitterState {
                    period: Duration::from_micros(500),
                    movement_spec: Some(ParticleMovementSpec {
                        angle_range: ParticleAngleRange::all(),
                        cardinal_period_range: DurationRange {
                            min: Duration::from_millis(200),
                            max: Duration::from_millis(500),
                        },
                    }),
                    particle_fade_spec: ParticleFadeSpec {
                        initial_progress: ParticleInitialFadeProgress::Zero,
                        full_duration: Duration::from_millis(1000),
                    },
                    tile: Tile::Smoke,
                    fade_out_state: None,
                    colour_spec: None,
                    light_spec: None,
                    light_colour_fade_spec: None,
                    particle_emitter_spec: None,
                },
                until_next_event: Duration::from_millis(0),
            },
        );
        self.ecs.components.tile.insert(bullet_entity, Tile::Bullet);
        self.ecs
            .components
            .on_collision
            .insert(bullet_entity, OnCollision::Explode);
        self.ecs.components.light.insert(
            bullet_entity,
            Light {
                colour: Rgb24::new(255, 187, 63),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
    }
    pub fn opacity(&self, coord: Coord) -> u8 {
        self.spatial_grid
            .get(coord)
            .and_then(|c| c.feature)
            .and_then(|e| self.ecs.components.opacity.get(e).cloned())
            .unwrap_or(0)
    }
    pub fn entity_coord(&self, entity: Entity) -> Coord {
        self.ecs.components.location.get(entity).unwrap().coord
    }
    pub fn entity_npc(&self, entity: Entity) -> &Npc {
        self.ecs.components.npc.get(entity).unwrap()
    }
    pub fn size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        !self.ecs.components.blocks_gameplay.is_empty()
    }

    pub fn animation_tick<R: Rng>(&mut self, external_events: &mut Vec<ExternalEvent>, rng: &mut R) {
        self.realtime_entities.extend(self.ecs.components.realtime.entities());
        for entity in self.realtime_entities.drain(..) {
            let mut frame_remaining = FRAME_DURATION;
            while frame_remaining > Duration::from_micros(0) {
                let mut realtime_entity_components = self.realtime_components.get_mut_of_entity(entity);
                let TimeConsumingEvent {
                    event,
                    until_next_event,
                } = realtime_entity_components.tick(frame_remaining, rng);
                frame_remaining -= until_next_event;
                event.animate(
                    &mut self.ecs,
                    &mut self.realtime_components,
                    &mut self.spatial_grid,
                    entity,
                    external_events,
                );
            }
        }
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        let tile_component = &self.ecs.components.tile;
        let location_component = &self.ecs.components.location;
        let realtime_fade_component = &self.realtime_components.fade;
        let colour_hint_component = &self.ecs.components.colour_hint;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            if let Some(location) = location_component.get(entity) {
                let fade = realtime_fade_component.get(entity).and_then(|f| f.state.fading());
                let colour_hint = colour_hint_component.get(entity).cloned();
                Some(ToRenderEntity {
                    coord: location.coord,
                    layer: location.layer,
                    tile,
                    fade,
                    colour_hint,
                })
            } else {
                None
            }
        })
    }
    pub fn lights<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &'a Light)> {
        self.ecs.components.light.iter().filter_map(move |(entity, light)| {
            self.ecs
                .components
                .location
                .get(entity)
                .map(|location| (location.coord, light))
        })
    }
    pub fn contains_wall(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.feature {
                self.ecs.components.tile.get(entity) == Some(&Tile::Wall)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn contains_npc(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.character {
                self.ecs.components.npc.contains(entity)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn character_at(&self, coord: Coord) -> Option<Entity> {
        self.spatial_grid.get(coord).and_then(|cell| cell.character)
    }
}

pub struct ToRenderEntity {
    pub coord: Coord,
    pub layer: Layer,
    pub tile: Tile,
    pub fade: Option<u8>,
    pub colour_hint: Option<Rgb24>,
}
