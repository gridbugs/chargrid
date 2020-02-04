use crate::{world::World, ExternalEvent};
use ecs::Entity;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub trait RealtimePeriodicState {
    type Event;
    type Components;
    fn tick<R: Rng>(&mut self, rng: &mut R) -> TimeConsumingEvent<Self::Event>;
    fn animate_event(event: Self::Event, entity: Entity, world: &mut World, external_events: &mut Vec<ExternalEvent>);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledRealtimePeriodicState<S: RealtimePeriodicState> {
    pub state: S,
    pub until_next_event: Duration,
}

pub struct TimeConsumingEvent<E> {
    pub event: E,
    pub until_next_event: Duration,
}

#[macro_export]
macro_rules! realtime_periodic {
    { $module_name:ident { $($component_name:ident: $component_type:ty,)* } } => {
        mod $module_name {
            #[allow(unused_imports)]
            use super::*;
            use $crate::world::{
                World,
                realtime_periodic::core::{RealtimePeriodicState, ScheduledRealtimePeriodicState, TimeConsumingEvent}};

            ecs::ecs_components! {
                components {
                    $($component_name: ScheduledRealtimePeriodicState<$component_type>,)*
                }
            }
            pub use components::Components as RealtimeComponents;

            pub struct RealtimeEntityEvents {
                $(pub $component_name: Option<<$component_type as RealtimePeriodicState>::Event>,)*
            }

            impl RealtimeEntityEvents {
                pub fn animate(
                    self,
                    entity: ecs::Entity,
                    world: &mut World,
                    external_events: &mut Vec<crate::ExternalEvent>,
                ) {
                    $(if let Some(event) = self.$component_name {
                        <$component_type as RealtimePeriodicState>::animate_event(
                            event,
                            entity,
                            world,
                            external_events,
                        );
                    })*
                }
            }

            pub struct RealtimeEntityComponents<'a> {
                $($component_name: Option<&'a mut ScheduledRealtimePeriodicState<$component_type>>,)*
            }

            impl RealtimeComponents {
                pub fn get_mut_of_entity(&mut self, entity: Entity) -> RealtimeEntityComponents {
                    RealtimeEntityComponents {
                        $($component_name: self.$component_name.get_mut(entity),)*
                    }
                }
            }

            impl<'a> RealtimeEntityComponents<'a> {
                pub fn tick<R: Rng>(&mut self, frame_remaining: Duration, rng: &mut R) -> TimeConsumingEvent<RealtimeEntityEvents> {
                    let mut until_next_event = frame_remaining;
                    $(if let Some(event) = self.$component_name.as_ref() {
                        until_next_event = until_next_event.min(event.until_next_event);
                    })*
                    $(let $component_name = if let Some(component) = self.$component_name.as_mut() {
                        if until_next_event == component.until_next_event {
                            let TimeConsumingEvent {
                                until_next_event,
                                event,
                            } = component.state.tick(rng);
                            component.until_next_event = until_next_event;
                            Some(event)
                        } else {
                            component.until_next_event -= until_next_event;
                            None
                        }
                    } else {
                        None
                    };)*
                    TimeConsumingEvent {
                        until_next_event,
                        event: RealtimeEntityEvents {
                            $($component_name,)*
                        }
                    }
                }
            }
        }
    }
}
