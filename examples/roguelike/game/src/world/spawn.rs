use crate::{
    visibility::Light,
    world::{
        data::{ColidesWith, Components, Disposition, HitPoints, Layer, Location, Npc, OnCollision, Tile},
        realtime_periodic::{
            core::ScheduledRealtimePeriodicState,
            data::{period_per_frame, FadeState, LightColourFadeState, RealtimeComponents},
            movement, particle,
        },
        spatial_grid::{LocationUpdate, SpatialGrid},
    },
    ExternalEvent,
};
use ecs::{Ecs, Entity};
use grid_2d::Coord;
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

struct Explosion {
    duration: Duration,
    num_particles_per_frame: u32,
    min_step: Duration,
    max_step: Duration,
    fade_duration: Duration,
}

fn explosion_emitter(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    coord: Coord,
    Explosion {
        duration,
        num_particles_per_frame,
        min_step,
        max_step,
        fade_duration,
    }: Explosion,
) {
    let emitter_entity = ecs.entity_allocator.alloc();
    spatial_grid
        .location_update(
            ecs,
            emitter_entity,
            Location {
                coord,
                layer: Layer::Untracked,
            },
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
                        possible_damage: Some(Possible {
                            chance: Rational {
                                numerator: 1,
                                denominator: 20,
                            },
                            value: Damage {
                                range: DamageRange { min: 1, max: 5 },
                                push_back: true,
                            },
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
    spatial_grid: &mut SpatialGrid,
    coord: Coord,
    external_events: &mut Vec<ExternalEvent>,
) {
    explosion_emitter(
        ecs,
        realtime_components,
        spatial_grid,
        coord,
        Explosion {
            duration: Duration::from_millis(250),
            num_particles_per_frame: 50,
            min_step: Duration::from_millis(10),
            max_step: Duration::from_millis(30),
            fade_duration: Duration::from_millis(250),
        },
    );
    external_events.push(ExternalEvent::Explosion(coord));
}

pub fn player(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Character,
            },
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
    ecs.components.character.insert(entity, ());
    ecs.components.hit_points.insert(entity, HitPoints::new_full(1));
    ecs.components.player.insert(entity, ());
    entity
}

pub fn wall(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Feature,
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Wall);
    ecs.components.solid.insert(entity, ());
    ecs.components.opacity.insert(entity, 255);
    entity
}

pub fn former_human(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Character,
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::FormerHuman);
    ecs.components.npc.insert(
        entity,
        Npc {
            disposition: Disposition::Hostile,
        },
    );
    ecs.components.character.insert(entity, ());
    ecs.components.hit_points.insert(entity, HitPoints::new_full(20));
    entity
}

pub fn human(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Character,
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Human);
    ecs.components.npc.insert(
        entity,
        Npc {
            disposition: Disposition::Afraid,
        },
    );
    ecs.components.character.insert(entity, ());
    ecs.components.hit_points.insert(entity, HitPoints::new_full(20));
    entity
}

pub fn floor(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Floor,
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Floor);
    entity
}

pub fn carpet(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Floor,
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Carpet);
    entity
}

pub fn light(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord, colour: Rgb24) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord,
                layer: Layer::Feature,
            },
        )
        .unwrap();
    ecs.components.light.insert(
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

pub fn rocket(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    start: Coord,
    target: Coord,
) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .location_update(
            ecs,
            entity,
            Location {
                coord: start,
                layer: Layer::Untracked,
            },
        )
        .unwrap();
    ecs.components.realtime.insert(entity, ());
    ecs.components.blocks_gameplay.insert(entity, ());
    realtime_components.movement.insert(
        entity,
        ScheduledRealtimePeriodicState {
            state: movement::spec::Movement {
                path: target - start,
                cardinal_step_duration: Duration::from_millis(16),
                infinite: false,
            }
            .build(),
            until_next_event: Duration::from_millis(0),
        },
    );
    realtime_components.particle_emitter.insert(
        entity,
        ScheduledRealtimePeriodicState {
            state: {
                use particle::spec::*;
                ParticleEmitter {
                    emit_particle_every_period: Duration::from_micros(500),
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
                }
                .build()
            },
            until_next_event: Duration::from_millis(0),
        },
    );
    ecs.components.tile.insert(entity, Tile::Bullet);
    ecs.components.on_collision.insert(entity, OnCollision::Explode);
    ecs.components.light.insert(
        entity,
        Light {
            colour: Rgb24::new(255, 187, 63),
            vision_distance: Circle::new_squared(90),
            diminish: Rational {
                numerator: 1,
                denominator: 10,
            },
        },
    );
    ecs.components.colides_with.insert(
        entity,
        ColidesWith {
            solid: true,
            character: true,
        },
    );
    entity
}
