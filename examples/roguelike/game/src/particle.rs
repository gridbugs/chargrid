use crate::rational::Rational;
use crate::realtime_periodic_core::{ScheduledRealtimePeriodicState, TimeConsumingEvent};
use crate::realtime_periodic_data::{FadeProgress, FadeState, LightColourFadeState, MovementState, RealtimeComponents};
use crate::visibility::Light;
use crate::world_data::{location_insert, Components, Layer, Location, SpatialCell, Tile};
use crate::ExternalEvent;
use ecs::{Ecs, Entity};
use grid_2d::Grid;
use line_2d::InfiniteStepIter;
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use vector::Radial;

pub struct Particle {
    movement: Option<MovementState>,
    fade_state: FadeState,
    tile: Tile,
    colour_hint: Option<Rgb24>,
    light: Option<Light>,
    light_colour_fade: Option<LightColourFadeState>,
    particle_emitter: Option<Box<ParticleEmitterState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterState {
    pub period: Duration,
    pub movement_spec: Option<ParticleMovementSpec>,
    pub particle_fade_spec: ParticleFadeSpec,
    pub tile: Tile,
    pub fade_out_state: Option<ParticleEmitterFadeOutState>,
    pub colour_spec: Option<ParticleColourSpec>,
    pub light_spec: Option<ParticleLightSpec>,
    pub light_colour_fade_spec: Option<ParticleLightColourFadeSpec>,
    pub particle_emitter_spec: Option<ParticleEmitterSpec>,
}

impl ParticleEmitterState {
    fn emit<R: Rng>(&self, fade_out_progress: Option<FadeProgress>, rng: &mut R) -> Particle {
        let fade_state = match self.particle_fade_spec.initial_progress {
            ParticleInitialFadeProgress::Zero => FadeState::new(self.particle_fade_spec.full_duration),
            ParticleInitialFadeProgress::FromEmitter => FadeState::new_with_progress(
                self.particle_fade_spec.full_duration,
                fade_out_progress.unwrap_or_default(),
            ),
        };
        let light_colour_fade = self.light_colour_fade_spec.as_ref().map(|spec| {
            let fade_state = match spec.fade_spec.initial_progress {
                ParticleInitialFadeProgress::Zero => FadeState::new(self.particle_fade_spec.full_duration),
                ParticleInitialFadeProgress::FromEmitter => FadeState::new_with_progress(
                    self.particle_fade_spec.full_duration,
                    fade_out_progress.unwrap_or_default(),
                ),
            };
            LightColourFadeState {
                fade_state,
                from: spec.from,
                to: spec.to,
            }
        });
        Particle {
            movement: self.movement_spec.as_ref().map(|s| s.movement(rng)),
            fade_state,
            tile: self.tile,
            colour_hint: self.colour_spec.map(|c| c.choose(rng)),
            light: self.light_spec.as_ref().and_then(|l| l.choose(rng)),
            light_colour_fade,
            particle_emitter: self.particle_emitter_spec.as_ref().and_then(|p| p.choose(rng)),
        }
    }
    pub fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Particle> {
        let until_next_event = self.period;
        let fade_out_progress = self
            .fade_out_state
            .as_mut()
            .map(|fade_out_state| fade_out_state.fade(until_next_event));
        TimeConsumingEvent {
            event: self.emit(fade_out_progress, rng),
            until_next_event,
        }
    }
    pub fn animate_event(
        mut particle: Particle,
        ecs: &mut Ecs<Components>,
        realtime_components: &mut RealtimeComponents,
        spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        _external_events: &mut Vec<ExternalEvent>,
    ) {
        let coord = if let Some(location) = ecs.components.location.get(entity) {
            location.coord
        } else {
            return;
        };
        let particle_entity = ecs.entity_allocator.alloc();
        if let Some(movement) = particle.movement.take() {
            realtime_components.movement.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    until_next_event: movement.cardinal_period(),
                    state: movement,
                },
            );
        }
        location_insert(
            particle_entity,
            Location::new(coord, Layer::Particle),
            &mut ecs.components.location,
            spatial_grid,
        )
        .unwrap();
        ecs.components.tile.insert(particle_entity, particle.tile);
        realtime_components.fade.insert(
            particle_entity,
            ScheduledRealtimePeriodicState {
                state: particle.fade_state,
                until_next_event: Duration::from_millis(0),
            },
        );
        ecs.components.realtime.insert(particle_entity, ());
        if let Some(colour_hint) = particle.colour_hint {
            ecs.components.colour_hint.insert(particle_entity, colour_hint);
        }
        if let Some(light) = particle.light.take() {
            ecs.components.light.insert(particle_entity, light);
        }
        if let Some(light_colour_fade) = particle.light_colour_fade.take() {
            realtime_components.light_colour_fade.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    state: light_colour_fade,
                    until_next_event: Duration::from_millis(0),
                },
            );
        }
        if let Some(particle_emitter) = particle.particle_emitter.take() {
            realtime_components.particle_emitter.insert(
                particle_entity,
                ScheduledRealtimePeriodicState {
                    state: *particle_emitter,
                    until_next_event: Duration::from_millis(0),
                },
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleAngleRange {
    pub min: f64,
    pub max: f64,
}

impl ParticleAngleRange {
    pub fn all() -> Self {
        Self {
            min: -::std::f64::consts::PI,
            max: ::std::f64::consts::PI,
        }
    }
    fn choose<R: Rng>(&self, rng: &mut R) -> f64 {
        rng.gen_range(self.min, self.max)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationRange {
    pub min: Duration,
    pub max: Duration,
}

impl DurationRange {
    fn choose<R: Rng>(&self, rng: &mut R) -> Duration {
        rng.gen_range(self.min, self.max)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleMovementSpec {
    pub angle_range: ParticleAngleRange,
    pub cardinal_period_range: DurationRange,
}

impl ParticleMovementSpec {
    fn movement<R: Rng>(&self, rng: &mut R) -> MovementState {
        const VECTOR_LENGTH: f64 = 1000.;
        let angle_radians = self.angle_range.choose(rng);
        let radial = Radial {
            angle_radians,
            length: VECTOR_LENGTH,
        };
        let delta = radial.to_cartesian().to_coord_round_nearest();
        let path = InfiniteStepIter::new(delta);
        let cardinal_period = self.cardinal_period_range.choose(rng);
        MovementState::new(path, cardinal_period)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ParticleInitialFadeProgress {
    Zero,
    FromEmitter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleFadeSpec {
    pub initial_progress: ParticleInitialFadeProgress,
    pub full_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleLightColourFadeSpec {
    pub fade_spec: ParticleFadeSpec,
    pub from: Rgb24,
    pub to: Rgb24,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParticleColourSpec {
    pub from: Rgb24,
    pub to: Rgb24,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleLightSpec {
    pub chance: Rational,
    pub light: Light,
}

impl ParticleLightSpec {
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<Light> {
        if self.chance.roll(rng) {
            Some(self.light.clone())
        } else {
            None
        }
    }
}

impl ParticleColourSpec {
    fn choose<R: Rng>(self, rng: &mut R) -> Rgb24 {
        self.from.linear_interpolate(self.to, rng.gen())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterSpec {
    pub chance: Rational,
    pub particle_emitter: Box<ParticleEmitterState>,
}

impl ParticleEmitterSpec {
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<Box<ParticleEmitterState>> {
        if self.chance.roll(rng) {
            Some(self.particle_emitter.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterFadeOutState {
    total: Duration,
    elapsed: Duration,
}

impl ParticleEmitterFadeOutState {
    pub fn new(total: Duration) -> Self {
        Self {
            total,
            elapsed: Duration::from_micros(0),
        }
    }
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
