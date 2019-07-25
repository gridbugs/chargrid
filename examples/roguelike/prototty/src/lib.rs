use event_routine::*;
use prototty::*;
use render::View;

pub struct AppData {}

impl AppData {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct AppView {}

impl AppView {
    pub fn new() -> Self {
        Self {}
    }
}

struct PressAnyKey;

impl EventRoutine for PressAnyKey {
    type Return = ();
    type Data = AppData;
    type View = AppView;
    type Event = CommonEvent;

    fn handle<EP>(self, _data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => {
                if input.is_key() {
                    Handled::Return(())
                } else {
                    Handled::Continue(s)
                }
            }
            CommonEvent::Frame(_) => Handled::Continue(s),
        })
    }

    fn view<F, R>(&self, _data: &Self::Data, _view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
        text::StringViewSingleLine::default().view("Press any key...", context.add_offset(Coord::new(1, 1)), frame);
    }
}

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    PressAnyKey
}
