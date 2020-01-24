use crate::{
    visibility::Light,
    world::{
        location_insert,
        realtime_periodic::{
            core::ScheduledRealtimePeriodicState,
            data::{period_per_frame, FadeState, LightColourFadeState, RealtimeComponents},
            particle,
        },
        Components, Layer, Location, SpatialCell, Tile,
    },
    ExternalEvent,
};
use ecs::{Ecs, Entity};
use grid_2d::{Coord, Grid};
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

fn explosion_emitter(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut Grid<SpatialCell>,
    coord: Coord,
    duration: Duration,
    num_particles_per_frame: u32,
    min_step: Duration,
    max_step: Duration,
    fade_duration: Duration,
) {
    let emitter_entity = ecs.entity_allocator.alloc();
    location_insert(
        emitter_entity,
        Location::new(coord, Layer::Particle),
        &mut ecs.components.location,
        spatial_grid,
    )
    .unwrap();
    realtime_components.fade.insert(
        emitter_entity,
        ScheduledRealtimePeriodicState {
            state: FadeState::new(duration),
            until_next_event: Duration::from_millis(0),
        },
    );
    ecs.components.realtime.insert(emitter_entity, ());
    realtime_components.particle_emitter.insert(
        emitter_entity,
        ScheduledRealtimePeriodicState {
            state: {
                use particle::spec::*;
                ParticleEmitter {
                    emit_particle_every_period: period_per_frame(num_particles_per_frame),
                    fade_out_duration: Some(duration),
                    particle: Particle {
                        tile: Some(Tile::ExplosionFlame),
                        movement: Some(Movement {
                            angle_range: AngleRange::all(),
                            cardinal_period_range: DurationRange {
                                min: min_step,
                                max: max_step,
                            },
                        }),
                        fade_duration: Some(fade_duration),
                        colour_hint: Some(ColourRange {
                            from: Rgb24::new(255, 255, 63),
                            to: Rgb24::new(255, 127, 0),
                        }),
                        possible_particle_emitter: Some(Possible {
                            chance: Rational {
                                numerator: 1,
                                denominator: 20,
                            },
                            value: Box::new(ParticleEmitter {
                                emit_particle_every_period: min_step,
                                fade_out_duration: None,
                                particle: Particle {
                                    tile: Some(Tile::Smoke),
                                    movement: Some(Movement {
                                        angle_range: AngleRange::all(),
                                        cardinal_period_range: DurationRange {
                                            min: Duration::from_millis(200),
                                            max: Duration::from_millis(500),
                                        },
                                    }),
                                    fade_duration: Some(Duration::from_millis(1000)),
                                    ..Default::default()
                                },
                            }),
                        }),
                        ..Default::default()
                    },
                }
                .build()
            },
            until_next_event: Duration::from_millis(0),
        },
    );
    ecs.components.light.insert(
        emitter_entity,
        Light {
            colour: Rgb24::new(255, 187, 63),
            vision_distance: Circle::new_squared(420),
            diminish: Rational {
                numerator: 1,
                denominator: 100,
            },
        },
    );
    realtime_components.light_colour_fade.insert(
        emitter_entity,
        ScheduledRealtimePeriodicState {
            state: LightColourFadeState {
                fade_state: FadeState::new(fade_duration),
                from: Rgb24::new(255, 187, 63),
                to: Rgb24::new(0, 0, 0),
            },
            until_next_event: Duration::from_millis(0),
        },
    );
}

pub fn explosion(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut Grid<SpatialCell>,
    coord: Coord,
    external_events: &mut Vec<ExternalEvent>,
) {
    explosion_emitter(
        ecs,
        realtime_components,
        spatial_grid,
        coord,
        Duration::from_millis(250),
        50,
        Duration::from_millis(10),
        Duration::from_millis(30),
        Duration::from_millis(250),
    );
    external_events.push(ExternalEvent::Explosion(coord));
}

pub fn player(ecs: &mut Ecs<Components>, spatial_grid: &mut Grid<SpatialCell>, coord: Coord) -> Entity {
    let entity = ecs.create();
    location_insert(
        entity,
        Location::new(coord, Layer::Character),
        &mut ecs.components.location,
        spatial_grid,
    )
    .unwrap();
    ecs.components.tile.insert(entity, Tile::Player);
    ecs.components.light.insert(
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
