use crate::{
    particle::{Particle, ParticleEmitterState},
    realtime_periodic_core::{RealtimePeriodicState, TimeConsumingEvent},
    spawn,
    world_data::{is_solid_feature_at_coord, Components, OnCollision, SpatialCell},
    ExternalEvent,
};
use direction::Direction;
use ecs::{Ecs, Entity};
use grid_2d::Grid;
use line_2d::InfiniteStepIter;
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

pub fn period_per_frame(num_per_frame: u32) -> Duration {
    FRAME_DURATION / num_per_frame
}

crate::realtime_periodic! {
    realtime_periodic {
        movement: MovementState,
        particle_emitter: ParticleEmitterState,
        fade: FadeState,
        light_colour_fade: LightColourFadeState,
    }
}

pub use realtime_periodic::RealtimeComponents;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementState {
    path: InfiniteStepIter,
    cardinal_period: Duration,
    ordinal_period: Duration,
}

impl MovementState {
    fn ordinal_duration_from_cardinal_duration(duration: Duration) -> Duration {
        const SQRT_2_X_1_000_000: u64 = 1_414_214;
        let ordinal_micros = (duration.as_micros() as u64 * SQRT_2_X_1_000_000) / 1_000_000;
        Duration::from_micros(ordinal_micros)
    }

    pub fn new(path: InfiniteStepIter, cardinal_period: Duration) -> Self {
        Self {
            path,
            cardinal_period,
            ordinal_period: Self::ordinal_duration_from_cardinal_duration(cardinal_period),
        }
    }

    pub fn cardinal_period(&self) -> Duration {
        self.cardinal_period
    }
}

impl RealtimePeriodicState for MovementState {
    type Event = Direction;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, _rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let direction = self.path.step();
        let until_next_event = if direction.is_cardinal() {
            self.cardinal_period
        } else {
            self.ordinal_period
        };
        TimeConsumingEvent {
            event: direction,
            until_next_event,
        }
    }
    fn animate_event(
        movement_direction: Self::Event,
        ecs: &mut Ecs<Components>,
        realtime_components: &mut Self::Components,
        spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        external_events: &mut Vec<ExternalEvent>,
    ) {
        if let Some(current_location) = ecs.components.location.get_mut(entity) {
            let next_coord = current_location.coord + movement_direction.coord();
            if is_solid_feature_at_coord(next_coord, &ecs.components.solid, spatial_grid) {
                if let Some(on_collision) = ecs.components.on_collision.get(entity) {
                    let current_coord = current_location.coord;
                    match on_collision {
                        OnCollision::Explode => {
                            spawn::explosion(ecs, realtime_components, spatial_grid, current_coord, external_events);
                        }
                    }
                }
                ecs.remove(entity);
            } else {
                current_location.coord += movement_direction.coord();
            }
        } else {
            ecs.remove(entity);
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FadeProgress {
    Fading(u8),
    Complete,
}

impl FadeProgress {
    fn fading(self) -> Option<u8> {
        match self {
            Self::Fading(progress) => Some(progress),
            Self::Complete => None,
        }
    }
    fn is_complete(self) -> bool {
        match self {
            Self::Fading(_) => false,
            Self::Complete => true,
        }
    }
}

impl Default for FadeProgress {
    fn default() -> Self {
        Self::Fading(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FadeState {
    progress: FadeProgress,
    period: Duration,
}

impl FadeState {
    pub fn new(duration: Duration) -> Self {
        Self::new_with_progress(duration, FadeProgress::default())
    }
    pub fn new_with_progress(full_duration: Duration, progress: FadeProgress) -> Self {
        let period = full_duration / 256;
        Self { progress, period }
    }
    pub fn fading(self) -> Option<u8> {
        self.progress.fading()
    }
}

impl RealtimePeriodicState for FadeState {
    type Event = FadeProgress;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, _rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        self.progress = match self.progress {
            FadeProgress::Complete => FadeProgress::Complete,
            FadeProgress::Fading(progress) => match progress.checked_add(1) {
                Some(progress) => FadeProgress::Fading(progress),
                None => FadeProgress::Complete,
            },
        };
        TimeConsumingEvent {
            event: self.progress,
            until_next_event: self.period,
        }
    }
    fn animate_event(
        progress: Self::Event,
        ecs: &mut Ecs<Components>,
        _realtime_components: &mut Self::Components,
        _spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        _external_events: &mut Vec<ExternalEvent>,
    ) {
        if progress.is_complete() {
            ecs.remove(entity);
        }
    }
}

pub enum LightColourFadeProgress {
    Colour(Rgb24),
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightColourFadeState {
    pub fade_state: FadeState,
    pub from: Rgb24,
    pub to: Rgb24,
}

impl RealtimePeriodicState for LightColourFadeState {
    type Event = LightColourFadeProgress;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let TimeConsumingEvent {
            event: fade_progress,
            until_next_event,
        } = self.fade_state.tick(rng);
        let event = match fade_progress {
            FadeProgress::Complete => LightColourFadeProgress::Complete,
            FadeProgress::Fading(fading) => {
                LightColourFadeProgress::Colour(self.from.linear_interpolate(self.to, fading))
            }
        };
        TimeConsumingEvent {
            event,
            until_next_event,
        }
    }
    fn animate_event(
        progress: Self::Event,
        ecs: &mut Ecs<Components>,
        _realtime_components: &mut Self::Components,
        _spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        _external_events: &mut Vec<ExternalEvent>,
    ) {
        match progress {
            LightColourFadeProgress::Colour(colour) => {
                if let Some(light) = ecs.components.light.get_mut(entity) {
                    light.colour = colour;
                }
            }
            LightColourFadeProgress::Complete => {
                ecs.components.light.remove(entity);
            }
        }
    }
}

impl RealtimePeriodicState for ParticleEmitterState {
    type Event = Particle;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        ParticleEmitterState::tick(self, rng)
    }
    fn animate_event(
        particle: Self::Event,
        ecs: &mut Ecs<Components>,
        realtime_components: &mut Self::Components,
        spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        external_events: &mut Vec<ExternalEvent>,
    ) {
        ParticleEmitterState::animate_event(
            particle,
            ecs,
            realtime_components,
            spatial_grid,
            entity,
            external_events,
        );
    }
}
