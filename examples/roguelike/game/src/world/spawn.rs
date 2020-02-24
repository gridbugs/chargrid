use crate::{
    visibility::Light,
    world::{
        data::{CollidesWith, Disposition, DoorState, HitPoints, Layer, Location, Npc, OnCollision, Tile},
        explosion,
        realtime_periodic::{
            core::ScheduledRealtimePeriodicState,
            data::{period_per_frame, FadeState, LightColourFadeState},
            flicker, movement, particle,
        },
        World,
    },
};
use ecs::Entity;
use grid_2d::Coord;
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

impl World {
    pub fn spawn_player(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Player);
        self.ecs.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new(187, 187, 187),
                vision_distance: Circle::new_squared(70),
                diminish: Rational {
                    numerator: 1,
                    denominator: 1,
                },
            },
        );
        self.ecs.components.character.insert(entity, ());
        self.ecs.components.hit_points.insert(entity, HitPoints::new_full(100));
        self.ecs.components.player.insert(entity, ());
        entity
    }

    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Wall);
        self.ecs.components.solid.insert(entity, ());
        self.ecs.components.opacity.insert(entity, 255);
        entity
    }

    pub fn spawn_former_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::FormerHuman);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Hostile,
            },
        );
        self.ecs.components.character.insert(entity, ());
        self.ecs.components.hit_points.insert(entity, HitPoints::new_full(2));
        entity
    }

    pub fn spawn_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Human);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Afraid,
            },
        );
        self.ecs.components.character.insert(entity, ());
        self.ecs.components.hit_points.insert(entity, HitPoints::new_full(20));
        entity
    }

    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Floor);
        entity
    }

    pub fn spawn_carpet(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Carpet);
        entity
    }

    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.ecs.create();
        self.spatial.insert(entity, Location { coord, layer: None }).unwrap();
        self.ecs.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(200),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        entity
    }

    pub fn spawn_flickering_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.ecs.create();
        self.spatial.insert(entity, Location { coord, layer: None }).unwrap();
        self.ecs.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(200),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        self.ecs.components.realtime.insert(entity, ());
        self.realtime_components.flicker.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use flicker::spec::*;
                    Flicker {
                        colour_hint: None,
                        light_colour: Some(UniformInclusiveRange {
                            low: Rgb24::new(0, 0, 0),
                            high: colour,
                        }),
                        until_next_event: UniformInclusiveRange {
                            low: Duration::from_millis(127),
                            high: Duration::from_millis(512),
                        },
                    }
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        entity
    }

    pub fn spawn_flash(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial.insert(entity, Location { coord, layer: None }).unwrap();
        self.ecs.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new(127, 127, 127),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 20,
                },
            },
        );
        self.ecs.components.realtime.insert(entity, ());
        self.realtime_components.fade.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: FadeState::new(Duration::from_millis(32)),
                until_next_event: Duration::from_millis(0),
            },
        );
        entity
    }

    pub fn spawn_bullet(&mut self, start: Coord, target: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord: start,
                    layer: None,
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Bullet);
        self.ecs.components.realtime.insert(entity, ());
        self.ecs.components.blocks_gameplay.insert(entity, ());
        self.ecs.components.on_collision.insert(entity, OnCollision::Remove);
        self.realtime_components.movement.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: movement::spec::Movement {
                    path: target - start,
                    cardinal_step_duration: Duration::from_millis(12),
                    repeat: movement::spec::Repeat::Once,
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.realtime_components.particle_emitter.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use particle::spec::*;
                    ParticleEmitter {
                        emit_particle_every_period: Duration::from_micros(2000),
                        fade_out_duration: None,
                        particle: Particle {
                            tile: Some(Tile::Smoke),
                            movement: Some(Movement {
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: Duration::from_millis(200),
                                    high: Duration::from_millis(500),
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
        self.ecs.components.collides_with.insert(
            entity,
            CollidesWith {
                solid: true,
                character: true,
            },
        );
        entity
    }

    pub fn spawn_rocket(&mut self, start: Coord, target: Coord) -> Entity {
        let entity = self.ecs.create();
        self.spatial
            .insert(
                entity,
                Location {
                    coord: start,
                    layer: None,
                },
            )
            .unwrap();
        self.ecs.components.realtime.insert(entity, ());
        self.ecs.components.blocks_gameplay.insert(entity, ());
        self.realtime_components.movement.insert(
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
        self.realtime_components.particle_emitter.insert(
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
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: Duration::from_millis(200),
                                    high: Duration::from_millis(500),
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
        self.ecs.components.tile.insert(entity, Tile::Bullet);
        self.ecs.components.on_collision.insert(
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
        self.ecs.components.light.insert(
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
        self.ecs.components.collides_with.insert(
            entity,
            CollidesWith {
                solid: true,
                character: true,
            },
        );
        entity
    }

    pub fn spawn_explosion_emitter(&mut self, coord: Coord, spec: &explosion::spec::ParticleEmitter) {
        let emitter_entity = self.ecs.entity_allocator.alloc();
        self.spatial
            .insert(emitter_entity, Location { coord, layer: None })
            .unwrap();
        self.realtime_components.fade.insert(
            emitter_entity,
            ScheduledRealtimePeriodicState {
                state: FadeState::new(spec.duration),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.ecs.components.realtime.insert(emitter_entity, ());
        self.realtime_components.particle_emitter.insert(
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
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: spec.min_step,
                                    high: spec.max_step,
                                },
                            }),
                            fade_duration: Some(spec.fade_duration),
                            colour_hint: Some(UniformInclusiveRange {
                                low: Rgb24::new(255, 127, 0),
                                high: Rgb24::new(255, 255, 63),
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
                                            angle_range: Radians::uniform_range_all(),
                                            cardinal_period_range: UniformInclusiveRange {
                                                low: Duration::from_millis(200),
                                                high: Duration::from_millis(500),
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
        self.ecs.components.light.insert(
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
        self.realtime_components.light_colour_fade.insert(
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

    pub fn spawn_star(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.entity_allocator.alloc();
        self.spatial.insert(entity, Location { coord, layer: None }).unwrap();
        self.ecs.components.tile.insert(entity, Tile::Star);
        self.ecs.components.ignore_lighting.insert(entity, ());
        self.ecs.components.realtime.insert(entity, ());
        self.realtime_components.flicker.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use flicker::spec::*;
                    Flicker {
                        colour_hint: Some(UniformInclusiveRange {
                            low: Rgb24::new_grey(127),
                            high: Rgb24::new_grey(255),
                        }),
                        light_colour: None,
                        until_next_event: UniformInclusiveRange {
                            low: Duration::from_millis(64),
                            high: Duration::from_millis(512),
                        },
                    }
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        entity
    }

    pub fn spawn_space(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.entity_allocator.alloc();
        self.spatial.insert(entity, Location { coord, layer: None }).unwrap();
        self.ecs.components.tile.insert(entity, Tile::Space);
        self.ecs.components.ignore_lighting.insert(entity, ());
        entity
    }

    pub fn spawn_window(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.entity_allocator.alloc();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Window);
        self.ecs.components.solid.insert(entity, ());
        entity
    }

    pub fn spawn_door(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.entity_allocator.alloc();
        self.spatial
            .insert(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.ecs.components.tile.insert(entity, Tile::DoorClosed);
        self.ecs.components.opacity.insert(entity, 255);
        self.ecs.components.solid.insert(entity, ());
        self.ecs.components.door_state.insert(entity, DoorState::Closed);
        entity
    }
}
