use crate::{
    particle::{
        DurationRange, ParticleAngleRange, ParticleColourSpec, ParticleEmitterFadeOutState, ParticleEmitterSpec,
        ParticleEmitterState, ParticleFadeSpec, ParticleInitialFadeProgress, ParticleMovementSpec,
    },
    rational::Rational,
    realtime_periodic_core::ScheduledRealtimePeriodicState,
    realtime_periodic_data::{period_per_frame, FadeState, LightColourFadeState, RealtimeComponents},
    visibility::Light,
    world_data::{location_insert, Components, Layer, Location, SpatialCell, Tile},
    ExternalEvent,
};
use ecs::Ecs;
use grid_2d::{Coord, Grid};
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
            state: ParticleEmitterState {
                period: period_per_frame(num_particles_per_frame),
                movement_spec: Some(ParticleMovementSpec {
                    angle_range: ParticleAngleRange::all(),
                    cardinal_period_range: DurationRange {
                        min: min_step,
                        max: max_step,
                    },
                }),
                particle_fade_spec: ParticleFadeSpec {
                    initial_progress: ParticleInitialFadeProgress::FromEmitter,
                    full_duration: fade_duration,
                },
                tile: Tile::ExplosionFlame,
                colour_spec: Some(ParticleColourSpec {
                    from: Rgb24::new(255, 255, 63),
                    to: Rgb24::new(255, 127, 0),
                }),
                light_spec: None,
                fade_out_state: Some(ParticleEmitterFadeOutState::new(duration)),
                light_colour_fade_spec: None,
                particle_emitter_spec: Some(ParticleEmitterSpec {
                    chance: Rational {
                        numerator: 1,
                        denominator: 20,
                    },
                    particle_emitter: Box::new(ParticleEmitterState {
                        period: min_step,
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
                    }),
                }),
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
