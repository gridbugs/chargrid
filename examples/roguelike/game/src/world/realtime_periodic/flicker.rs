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
    pub use rand_range::UniformInclusiveRange;
    pub use rgb24::Rgb24;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Flicker {
        pub colour_hint: Option<UniformInclusiveRange<Rgb24>>,
        pub light_colour: Option<UniformInclusiveRange<Rgb24>>,
        pub until_next_event: UniformInclusiveRange<Duration>,
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
    colour_hint: Option<Rgb24>,
    light_colour: Option<Rgb24>,
}

impl RealtimePeriodicState for FlickerState {
    type Event = FlickerEvent;
    type Components = RealtimeComponents;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event> {
        let colour_hint = self.0.colour_hint.map(|r| r.choose(rng));
        let light_colour = self.0.light_colour.map(|r| r.choose(rng));
        let until_next_event = self.0.until_next_event.choose(rng);
        let event = FlickerEvent {
            colour_hint,
            light_colour,
        };
        TimeConsumingEvent {
            event,
            until_next_event,
        }
    }
    fn animate_event(event: Self::Event, entity: Entity, world: &mut World, _external_events: &mut Vec<ExternalEvent>) {
        if let Some(colour_hint) = event.colour_hint {
            world.ecs.components.colour_hint.insert(entity, colour_hint);
        }
        if let Some(light_colour) = event.light_colour {
            if let Some(light) = world.ecs.components.light.get_mut(entity) {
                light.colour = light_colour;
            }
        }
    }
}
