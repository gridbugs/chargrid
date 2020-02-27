use crate::visibility::Light;
use crate::{
    world::{
        data::{ProjectileDamage, Tile},
        realtime_periodic::{
            core::{RealtimePeriodicState, ScheduledRealtimePeriodicState, TimeConsumingEvent},
            data::{FadeProgress, FadeState, LightColourFadeState, RealtimeComponents},
            movement,
        },
        World,
    },
    ExternalEvent,
};
use ecs::Entity;
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use vector::Radial;

pub mod spec {
    pub use crate::{visibility::Light, world::Tile};
    pub use rand_range::{UniformInclusiveRange, UniformLeftInclusiveRange};
    pub use rational::Rational;
    pub use rgb24::Rgb24;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;
    pub use vector::Radians;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Possible<T: Clone> {
        pub chance: Rational,
        pub value: T,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Damage {
        pub range: UniformInclusiveRange<u32>,
        pub push_back: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Movement {
        pub angle_range: UniformLeftInclusiveRange<Radians>,
        pub cardinal_period_range: UniformInclusiveRange<Duration>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LightColourFade {
        pub duration: Duration,
        pub from: Rgb24,
        pub to: Rgb24,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Particle {
        pub fade_duration: Option<Duration>,
        pub tile: Option<Tile>,
        pub movement: Option<Movement>,
        pub colour_hint: Option<UniformInclusiveRange<Rgb24>>,
        pub light_colour_fade: Option<LightColourFade>,
        pub possible_light: Option<Possible<Light>>,
        pub possible_particle_emitter: Option<Possible<Box<ParticleEmitter>>>,
        pub possible_damage: Option<Possible<Damage>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ParticleEmitter {
        pub emit_particle_every_period: Duration,
        pub particle: Particle,
        pub fade_out_duration: Option<Duration>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FadeOutState {
    total: Duration,
    elapsed: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterState {
    emit_particle_every_period: Duration,
    particle_spec: spec::Particle,
    fade_out_state: Option<FadeOutState>,
}

pub struct SpawnParticle {
    movement_state: Option<movement::MovementState>,
    fade_state: Option<FadeState>,
    tile: Option<Tile>,
    colour_hint: Option<Rgb24>,
    light_colour_fade_state: Option<LightColourFadeState>,
    light: Option<Light>,
    particle_emitter: Option<Box<ParticleEmitterState>>,
    damage: Option<ProjectileDamage>,
}

impl<T: Clone> spec::Possible<T> {
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<T> {
        if self.chance.roll(rng) {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

impl spec::Movement {
    fn choose<R: Rng>(&self, rng: &mut R) -> movement::MovementState {
        const VECTOR_LENGTH: f64 = 1000.;
        let angle = self.angle_range.choose(rng);
        let radial = Radial {
            angle,
            length: VECTOR_LENGTH,
        };
        movement::spec::Movement {
            path: radial.to_cartesian().to_coord_round_nearest(),
            repeat: movement::spec::Repeat::Forever,
            cardinal_step_duration: self.cardinal_period_range.choose(rng),
        }
        .build()
    }
}

impl spec::ParticleEmitter {
    pub fn build(self) -> ParticleEmitterState {
        ParticleEmitterState {
            emit_particle_every_period: self.emit_particle_every_period,
            particle_spec: self.particle,
            fade_out_state: self.fade_out_duration.map(|d| FadeOutState {
                total: d,
                elapsed: Duration::from_millis(0),
            }),
        }
    }
}

impl FadeOutState {
    fn fade(&mut self, duration: Duration) -> FadeProgress {
        self.elapsed += duration;
        if self.elapsed > self.total {
            FadeProgress::Complete
        } else {
            let ratio = ((self.elapsed.as_nanos() * 256) / self.total.as_nanos()).min(255) as u8;
            FadeProgress::Fading(ratio)
        }
    }
}

impl RealtimePeriodicState for ParticleEmitterState {
    type Event = SpawnParticle;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let until_next_event = self.emit_particle_every_period;
        let (fade_state, light_colour_fade_state) = match self.fade_out_state.as_mut() {
            None => (
                self.particle_spec.fade_duration.map(|d| FadeState::new(d)),
                self.particle_spec.light_colour_fade.as_ref().map(|l| {
                    let fade_state = FadeState::new(l.duration);
                    LightColourFadeState {
                        fade_state,
                        from: l.from,
                        to: l.to,
                    }
                }),
            ),
            Some(fade_out_state) => {
                let fade_out_progress = fade_out_state.fade(until_next_event);
                (
                    self.particle_spec
                        .fade_duration
                        .map(|d| FadeState::new_with_progress(d, fade_out_progress)),
                    self.particle_spec.light_colour_fade.as_ref().map(|l| {
                        let fade_state = FadeState::new_with_progress(l.duration, fade_out_progress);
                        LightColourFadeState {
                            fade_state,
                            from: l.from,
                            to: l.to,
                        }
                    }),
                )
            }
        };
        let event = SpawnParticle {
            movement_state: self.particle_spec.movement.as_ref().map(|m| m.choose(rng)),
            fade_state,
            tile: self.particle_spec.tile,
            colour_hint: self.particle_spec.colour_hint.map(|c| c.choose(rng)),
            light_colour_fade_state,
            light: self.particle_spec.possible_light.as_ref().and_then(|l| l.choose(rng)),
            particle_emitter: self
                .particle_spec
                .possible_particle_emitter
                .as_ref()
                .and_then(|p| p.choose(rng).map(|p| Box::new(p.build()))),
            damage: self.particle_spec.possible_damage.as_ref().and_then(|d| {
                d.choose(rng).map(|d| ProjectileDamage {
                    hit_points: d.range.choose(rng),
                    push_back: d.push_back,
                })
            }),
        };
        TimeConsumingEvent {
            event,
            until_next_event,
        }
    }
    fn animate_event(mut spawn_particle: Self::Event, entity: Entity, world: &mut World, _: &mut Vec<ExternalEvent>) {
        let coord = if let Some(&coord) = world.spatial.coord(entity) {
            coord
        } else {
            return;
        };
        let particle_entity = world.entity_allocator.alloc();
        if let Some(movement) = spawn_particle.movement_state.take() {
            world.realtime_components.movement.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    until_next_event: movement.cardinal_step_duration(),
                    state: movement,
                },
            );
        }
        world.spatial.update_coord(particle_entity, coord).unwrap();
        if let Some(tile) = spawn_particle.tile {
            world.components.tile.insert(particle_entity, tile);
        }
        if let Some(fade_state) = spawn_particle.fade_state {
            world.realtime_components.fade.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    state: fade_state,
                    until_next_event: Duration::from_millis(0),
                },
            );
        }
        world.components.realtime.insert(particle_entity, ());
        if let Some(colour_hint) = spawn_particle.colour_hint {
            world.components.colour_hint.insert(particle_entity, colour_hint);
        }
        if let Some(light) = spawn_particle.light.take() {
            world.components.light.insert(particle_entity, light);
        }
        if let Some(light_colour_fade) = spawn_particle.light_colour_fade_state.take() {
            world.realtime_components.light_colour_fade.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    state: light_colour_fade,
                    until_next_event: Duration::from_millis(0),
                },
            );
        }
        if let Some(particle_emitter) = spawn_particle.particle_emitter.take() {
            world.realtime_components.particle_emitter.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    state: *particle_emitter,
                    until_next_event: Duration::from_millis(0),
                },
            );
        }
        if let Some(projectile_damage) = spawn_particle.damage.take() {
            world
                .components
                .projectile_damage
                .insert(particle_entity, projectile_damage);
        }
    }
}
