use crate::*;
use chargrid_input::keys;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommonEvent {
    Input(Input),
    Frame(Duration),
}

impl From<Input> for CommonEvent {
    fn from(input: Input) -> Self {
        CommonEvent::Input(input)
    }
}

impl From<Duration> for CommonEvent {
    fn from(duration: Duration) -> Self {
        CommonEvent::Frame(duration)
    }
}

pub struct ConvertInputToCommonEvent<T>(pub(crate) T);

impl<T> EventRoutine for ConvertInputToCommonEvent<T>
where
    T: EventRoutine<Event = Input>,
{
    type Return = T::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek.with(
            self,
            |s, event| match event {
                CommonEvent::Input(input) => s.0.handle(data, view, Event::new(input)).map_continue(Self),
                CommonEvent::Frame(_) => Handled::Continue(s),
            },
            Handled::Continue,
        )
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        self.0.view(data, view, context, frame)
    }
}

pub struct ReturnOnExit<T, F> {
    pub(crate) t: T,
    pub(crate) f: F,
}

impl<T, F> EventRoutine for ReturnOnExit<T, F>
where
    T: EventRoutine<Event = CommonEvent>,
    F: FnOnce(&mut T::Data) -> T::Return,
{
    type Return = T::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek.with(
            (self, data),
            |(s, data), event| {
                let Self { t, f } = s;
                if event == CommonEvent::Input(Input::Keyboard(keys::ETX)) {
                    Handled::Return(f(data))
                } else {
                    t.handle(data, view, Event::new(event)).map_continue(|t| Self { t, f })
                }
            },
            |(s, data)| {
                let Self { t, f } = s;
                t.handle(data, view, Peek::new()).map_continue(|t| Self { t, f })
            },
        )
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        self.t.view(data, view, context, frame)
    }
}
