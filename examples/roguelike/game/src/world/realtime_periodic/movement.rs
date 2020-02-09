use crate::{
    world::{
        realtime_periodic::{
            core::{RealtimePeriodicState, TimeConsumingEvent},
            data::RealtimeComponents,
        },
        World,
    },
    ExternalEvent,
};
use direction::Direction;
use ecs::Entity;
use line_2d::{InfiniteStepIter, StepIter};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod spec {
    pub use grid_2d::Coord;
    pub use std::time::Duration;

    pub enum Repeat {
        Once,
        Forever,
        Steps(usize),
    }

    pub struct Movement {
        pub path: Coord,
        pub repeat: Repeat,
        pub cardinal_step_duration: Duration,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Path {
    Forever(InfiniteStepIter),
    Once(StepIter),
    Steps {
        infinite_step_iter: InfiniteStepIter,
        remaining_steps: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementState {
    path: Path,
    cardinal_step_duration: Duration,
    ordinal_step_duration: Duration,
}

fn ordinal_duration_from_cardinal_duration(duration: Duration) -> Duration {
    const SQRT_2_X_1_000_000: u64 = 1_414_214;
    let ordinal_micros = (duration.as_micros() as u64 * SQRT_2_X_1_000_000) / 1_000_000;
    Duration::from_micros(ordinal_micros)
}

impl spec::Movement {
    pub fn build(self) -> MovementState {
        MovementState {
            path: match self.repeat {
                spec::Repeat::Forever => Path::Forever(InfiniteStepIter::new(self.path)),
                spec::Repeat::Once => Path::Once(StepIter::new(self.path)),
                spec::Repeat::Steps(n) => Path::Steps {
                    infinite_step_iter: InfiniteStepIter::new(self.path),
                    remaining_steps: n,
                },
            },
            cardinal_step_duration: self.cardinal_step_duration,
            ordinal_step_duration: ordinal_duration_from_cardinal_duration(self.cardinal_step_duration),
        }
    }
}

impl MovementState {
    pub fn cardinal_step_duration(&self) -> Duration {
        self.cardinal_step_duration
    }
}

impl RealtimePeriodicState for MovementState {
    type Event = Option<Direction>;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, _rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let event = match self.path {
            Path::Forever(ref mut path) => path.next(),
            Path::Once(ref mut path) => path.next(),
            Path::Steps {
                ref mut infinite_step_iter,
                ref mut remaining_steps,
            } => {
                if let Some(next_remaining_steps) = remaining_steps.checked_sub(1) {
                    *remaining_steps = next_remaining_steps;
                    infinite_step_iter.next()
                } else {
                    None
                }
            }
        };
        let until_next_event = if let Some(direction) = event {
            if direction.is_cardinal() {
                self.cardinal_step_duration
            } else {
                self.ordinal_step_duration
            }
        } else {
            self.cardinal_step_duration
        };
        TimeConsumingEvent {
            event,
            until_next_event,
        }
    }
    fn animate_event(event: Self::Event, entity: Entity, world: &mut World, external_events: &mut Vec<ExternalEvent>) {
        if let Some(movement_direction) = event {
            world.projectile_move(entity, movement_direction, external_events);
        } else {
            world.projectile_stop(entity, external_events);
        }
    }
}
