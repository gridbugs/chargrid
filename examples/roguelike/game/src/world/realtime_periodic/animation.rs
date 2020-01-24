use crate::{
    world::{
        data::Components,
        realtime_periodic::{
            core::TimeConsumingEvent,
            data::{RealtimeComponents, FRAME_DURATION},
        },
        spatial_grid::SpatialGrid,
    },
    ExternalEvent,
};
use ecs::{Ecs, Entity};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default)]
pub struct Context {
    realtime_entities: Vec<Entity>,
}

impl Serialize for Context {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        ().serialize(s)
    }
}

impl<'a> Deserialize<'a> for Context {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let () = Deserialize::deserialize(d)?;
        Ok(Self::default())
    }
}

impl Context {
    pub fn tick<R: Rng>(
        &mut self,
        ecs: &mut Ecs<Components>,
        realtime_components: &mut RealtimeComponents,
        spatial_grid: &mut SpatialGrid,
        external_events: &mut Vec<ExternalEvent>,
        rng: &mut R,
    ) {
        self.realtime_entities.extend(ecs.components.realtime.entities());
        for entity in self.realtime_entities.drain(..) {
            let mut frame_remaining = FRAME_DURATION;
            while frame_remaining > Duration::from_micros(0) {
                let mut realtime_entity_components = realtime_components.get_mut_of_entity(entity);
                let TimeConsumingEvent {
                    event,
                    until_next_event,
                } = realtime_entity_components.tick(frame_remaining, rng);
                frame_remaining -= until_next_event;
                event.animate(ecs, realtime_components, spatial_grid, entity, external_events);
            }
        }
    }
}
