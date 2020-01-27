use crate::{
    world::{
        action,
        data::Components,
        realtime_periodic::{
            core::{RealtimePeriodicState, TimeConsumingEvent},
            data::RealtimeComponents,
        },
        spatial_grid::SpatialGrid,
    },
    ExternalEvent,
};
use direction::Direction;
use ecs::{Ecs, Entity};
use line_2d::{InfiniteStepIter, StepIter};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod spec {
    pub use grid_2d::Coord;
    pub use std::time::Duration;
    pub struct Movement {
        pub path: Coord,
        pub infinite: bool,
        pub cardinal_step_duration: Duration,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Path {
    Infinite(InfiniteStepIter),
    Finite(StepIter),
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
            path: if self.infinite {
                Path::Infinite(InfiniteStepIter::new(self.path))
            } else {
                Path::Finite(StepIter::new(self.path))
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
            Path::Finite(ref mut path) => path.next(),
            Path::Infinite(ref mut path) => path.next(),
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
    fn animate_event(
        event: Self::Event,
        ecs: &mut Ecs<Components>,
        realtime_components: &mut Self::Components,
        spatial_grid: &mut SpatialGrid,
        entity: Entity,
        external_events: &mut Vec<ExternalEvent>,
    ) {
        if let Some(movement_direction) = event {
            action::projectile_move(
                ecs,
                realtime_components,
                spatial_grid,
                entity,
                movement_direction,
                external_events,
            )
        } else {
            action::projectile_stop(ecs, realtime_components, spatial_grid, entity, external_events)
        }
    }
}
