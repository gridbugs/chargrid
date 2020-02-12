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
use ecs::Entity;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod spec {
    pub use crate::util::range::{DurationRange, Rgb24Range};
    pub use rgb24::Rgb24;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Flicker {
        pub colour_hint: Rgb24Range,
        pub until_next_event: DurationRange,
    }
}

impl spec::Flicker {
    pub fn build(self) -> FlickerState {
        FlickerState(self)
    }
}

use rgb24::Rgb24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlickerState(spec::Flicker);

pub struct FlickerEvent {
    colour_hint: Rgb24,
}

impl RealtimePeriodicState for FlickerState {
    type Event = FlickerEvent;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let colour_hint = self.0.colour_hint.choose(rng);
        let until_next_event = self.0.until_next_event.choose(rng);
        let event = FlickerEvent { colour_hint };
        TimeConsumingEvent {
            event,
            until_next_event,
        }
    }
    fn animate_event(event: Self::Event, entity: Entity, world: &mut World, _external_events: &mut Vec<ExternalEvent>) {
        world.ecs.components.colour_hint.insert(entity, event.colour_hint);
    }
}
