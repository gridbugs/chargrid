use crate::{
    visibility::Light,
    world::{
        data::{ColidesWith, Components, Disposition, HitPoints, Layer, Location, Npc, OnCollision, Tile},
        explosion,
        realtime_periodic::{
            core::ScheduledRealtimePeriodicState,
            data::{period_per_frame, FadeState, LightColourFadeState, RealtimeComponents},
            movement, particle,
        },
        spatial_grid::{LocationUpdate, SpatialGrid},
    },
};
use ecs::{Ecs, Entity};
use grid_2d::Coord;
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

pub fn explosion_emitter(
    ecs: &mut Ecs<Components>,
    realtime_components: &mut RealtimeComponents,
    spatial_grid: &mut SpatialGrid,
    coord: Coord,
    spec: &explosion::spec::ParticleEmitter,
) {
    let emitter_entity = ecs.entity_allocator.alloc();
    spatial_grid
        .update_entity_location(ecs, emitter_entity, Location { coord, layer: None })
        .unwrap();
    realtime_components.fade.insert(
        emitter_entity,
        ScheduledRealtimePeriodicState {
            state: FadeState::new(spec.duration),
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
                    emit_particle_every_period: period_per_frame(spec.num_particles_per_frame),
                    fade_out_duration: Some(spec.duration),
                    particle: Particle {
                        tile: Some(Tile::ExplosionFlame),
                        movement: Some(Movement {
                            angle_range: AngleRange::all(),
                            cardinal_period_range: DurationRange {
                                min: spec.min_step,
                                max: spec.max_step,
                            },
                        }),
                        fade_duration: Some(spec.fade_duration),
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
                                emit_particle_every_period: spec.min_step,
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
                fade_state: FadeState::new(spec.fade_duration),
                from: Rgb24::new(255, 187, 63),
                to: Rgb24::new(0, 0, 0),
            },
            until_next_event: Duration::from_millis(0),
        },
    );
}

pub fn player(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Character),
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
    ecs.components.hit_points.insert(entity, HitPoints::new_full(100));
    ecs.components.player.insert(entity, ());
    entity
}

pub fn wall(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Feature),
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
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Character),
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
    ecs.components.hit_points.insert(entity, HitPoints::new_full(2));
    entity
}

pub fn human(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Character),
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
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Floor),
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Floor);
    entity
}

pub fn carpet(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Floor),
            },
        )
        .unwrap();
    ecs.components.tile.insert(entity, Tile::Carpet);
    entity
}

pub fn light(ecs: &mut Ecs<Components>, spatial_grid: &mut SpatialGrid, coord: Coord, colour: Rgb24) -> Entity {
    let entity = ecs.create();
    spatial_grid
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord,
                layer: Some(Layer::Feature),
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
        .update_entity_location(
            ecs,
            entity,
            Location {
                coord: start,
                layer: None,
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
                repeat: movement::spec::Repeat::Once,
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
    ecs.components.on_collision.insert(
        entity,
        OnCollision::Explode({
            use explosion::spec::*;
            Explosion {
                mechanics: Mechanics { range: 10 },
                particle_emitter: ParticleEmitter {
                    duration: Duration::from_millis(250),
                    num_particles_per_frame: 50,
                    min_step: Duration::from_millis(10),
                    max_step: Duration::from_millis(30),
                    fade_duration: Duration::from_millis(250),
                },
            }
        }),
    );
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
