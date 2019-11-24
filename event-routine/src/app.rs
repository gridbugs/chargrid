use crate::{common_event::CommonEvent, Event, EventRoutine, Handled};
use prototty_app::{App, ColModify, ControlFlow, Duration, Frame, Input, ViewContext};

pub struct EventRoutineOneShotIgnoreReturn<ER>
where
    ER: EventRoutine<Event = CommonEvent>,
{
    event_routine: Option<ER>,
    data: ER::Data,
    view: ER::View,
}

impl<ER> App for EventRoutineOneShotIgnoreReturn<ER>
where
    ER: EventRoutine<Event = CommonEvent>,
{
    fn on_input(&mut self, input: Input) -> Option<ControlFlow> {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            match event_routine.handle(&mut self.data, &self.view, Event::new(input.into())) {
                Handled::Continue(event_routine) => Some(event_routine),
                Handled::Return(_) => return Some(ControlFlow::Exit),
            }
        } else {
            log::warn!("app has exited");
            None
        };
        None
    }
    fn on_frame<F, C>(
        &mut self,
        since_last_frame: Duration,
        view_context: ViewContext<C>,
        frame: &mut F,
    ) -> Option<ControlFlow>
    where
        F: Frame,
        C: ColModify,
    {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            match event_routine.handle(&mut self.data, &self.view, Event::new(since_last_frame.into())) {
                Handled::Continue(event_routine) => {
                    event_routine.view(&self.data, &mut self.view, view_context, frame);
                    Some(event_routine)
                }
                Handled::Return(_) => return Some(ControlFlow::Exit),
            }
        } else {
            log::warn!("app has exited");
            None
        };
        None
    }
}
