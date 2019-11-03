pub use prototty_input;
pub use prototty_input::Input;
pub use prototty_render;
pub use prototty_render::{ColModify, Frame, View, ViewContext};
use std::marker::PhantomData;
use std::time::Duration;

pub mod common_event;

pub enum Handled<R, C> {
    Return(R),
    Continue(C),
}

impl<R, C> Handled<R, C> {
    pub fn map_continue<D, F>(self, f: F) -> Handled<R, D>
    where
        F: FnOnce(C) -> D,
    {
        match self {
            Handled::Return(r) => Handled::Return(r),
            Handled::Continue(c) => Handled::Continue(f(c)),
        }
    }
    pub fn map_return<S, F>(self, f: F) -> Handled<S, C>
    where
        F: FnOnce(R) -> S,
    {
        match self {
            Handled::Return(r) => Handled::Return(f(r)),
            Handled::Continue(c) => Handled::Continue(c),
        }
    }
}

pub trait EventOrPeek: Sized + private::Sealed {
    type Event;
    fn with<A, X, F, G>(self, arg: A, f: F, g: G) -> X
    where
        F: FnOnce(A, Self::Event) -> X,
        G: FnOnce(A) -> X;
}

pub fn event_or_peek_with_handled<EP, A, X, F>(event_or_peek: EP, arg: A, f: F) -> Handled<X, A>
where
    EP: EventOrPeek,
    F: FnOnce(A, EP::Event) -> Handled<X, A>,
{
    event_or_peek.with(arg, f, Handled::Continue)
}

pub struct Event<E>(E);
pub struct Peek<E>(PhantomData<E>);

impl<E> Event<E> {
    pub fn new(e: E) -> Self {
        Self(e)
    }
}

impl<E> Peek<E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E> EventOrPeek for Event<E> {
    type Event = E;
    fn with<A, X, F, G>(self, arg: A, f: F, _g: G) -> X
    where
        F: FnOnce(A, Self::Event) -> X,
        G: FnOnce(A) -> X,
    {
        f(arg, self.0)
    }
}

impl<E> EventOrPeek for Peek<E> {
    type Event = E;
    fn with<A, X, F, G>(self, arg: A, _f: F, g: G) -> X
    where
        F: FnOnce(A, Self::Event) -> X,
        G: FnOnce(A) -> X,
    {
        g(arg)
    }
}

mod private {
    pub trait Sealed {}
    impl<E> Sealed for super::Event<E> {}
    impl<E> Sealed for super::Peek<E> {}
}

pub trait EventRoutine: Sized {
    type Return;
    type Data;
    type View;
    type Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>;

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify;

    fn repeat<U, F>(self, f: F) -> Repeat<Self, F>
    where
        F: FnMut(Self::Return) -> Handled<U, Self>,
    {
        Repeat { t: self, f }
    }

    fn select<S>(self, selector: S) -> Select<Self, S>
    where
        S: Selector<DataOutput = Self::Data, ViewOutput = Self::View>,
    {
        Select { t: self, selector }
    }

    fn and_then<U, F>(self, f: F) -> AndThen<Self, U, F>
    where
        U: EventRoutine<Data = Self::Data, View = Self::View>,
        F: FnOnce(Self::Return) -> U,
    {
        AndThen(AndThenPrivate::First { t: self, f })
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Return) -> U,
    {
        Map { t: self, f }
    }

    fn convert_input_to_common_event(self) -> common_event::ConvertInputToCommonEvent<Self> {
        common_event::ConvertInputToCommonEvent(self)
    }

    fn return_on_exit<F>(self, f: F) -> common_event::ReturnOnExit<Self, F>
    where
        F: FnOnce(&mut Self::Data) -> Self::Return,
    {
        common_event::ReturnOnExit { t: self, f }
    }

    fn decorated<D>(self, d: D) -> Decorated<Self, D>
    where
        D: Decorate<View = Self::View, Data = Self::Data>,
    {
        Decorated { t: self, d }
    }

    fn on_event<F>(self, f: F) -> OnEvent<Self, F>
    where
        F: FnMut(&mut &mut Self::Data, &Self::Event),
    {
        OnEvent { t: self, f }
    }
}

struct OneShot<T, D, V, E> {
    field: T,
    data: PhantomData<D>,
    view: PhantomData<V>,
    event: PhantomData<E>,
}

impl<T, D, V, E> OneShot<T, D, V, E> {
    pub fn new(field: T) -> Self {
        Self {
            field,
            data: PhantomData,
            view: PhantomData,
            event: PhantomData,
        }
    }
}

pub struct Value<T, D, V, E>(OneShot<T, D, V, E>);

impl<T, D, V, E> Value<T, D, V, E> {
    pub fn new(value: T) -> Self {
        Self(OneShot::new(value))
    }
}

impl<T, D, V, E> EventRoutine for Value<T, D, V, E> {
    type Return = T;
    type Data = D;
    type View = V;
    type Event = E;

    fn handle<EP>(self, _data: &mut Self::Data, _view: &Self::View, _event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        Handled::Return(self.0.field)
    }

    fn view<F, C>(&self, _data: &Self::Data, _view: &mut Self::View, _context: ViewContext<C>, _frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
    }
}

pub struct SideEffect<F, D, V, E>(OneShot<F, D, V, E>);

impl<F, D, V, E> SideEffect<F, D, V, E> {
    pub fn new(f: F) -> Self {
        Self(OneShot::new(f))
    }
}

impl<F, D, V, E, T> EventRoutine for SideEffect<F, D, V, E>
where
    F: FnOnce(&mut D, &V) -> T,
{
    type Return = T;
    type Data = D;
    type View = V;
    type Event = E;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, _event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        Handled::Return((self.0.field)(data, view))
    }

    fn view<G, C>(&self, _data: &Self::Data, _view: &mut Self::View, _context: ViewContext<C>, _frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
    }
}

enum SideEffectThenPrivate<F, D, V, E, U> {
    First(OneShot<F, D, V, E>),
    Second(U),
}
pub struct SideEffectThen<F, D, V, E, U>(SideEffectThenPrivate<F, D, V, E, U>);

impl<F, D, V, E, U> SideEffectThen<F, D, V, E, U>
where
    U: EventRoutine<Data = D, View = V, Event = E>,
    F: FnOnce(&mut D, &V) -> U,
{
    pub fn new(f: F) -> Self {
        Self(SideEffectThenPrivate::First(OneShot::new(f)))
    }
}

impl<F, D, V, E, U> EventRoutine for SideEffectThen<F, D, V, E, U>
where
    U: EventRoutine<Data = D, View = V, Event = E>,
    F: FnOnce(&mut D, &V) -> U,
{
    type Return = U::Return;
    type Data = D;
    type View = V;
    type Event = E;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        match self.0 {
            SideEffectThenPrivate::First(one_shot) => (one_shot.field)(data, view)
                .handle(data, view, Peek::new())
                .map_continue(|u| SideEffectThen(SideEffectThenPrivate::Second(u))),
            SideEffectThenPrivate::Second(u) => u
                .handle(data, view, event_or_peek)
                .map_continue(|u| SideEffectThen(SideEffectThenPrivate::Second(u))),
        }
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        match self.0 {
            SideEffectThenPrivate::First(_) => (),
            SideEffectThenPrivate::Second(ref u) => u.view(data, view, context, frame),
        }
    }
}

pub struct Map<T, F> {
    t: T,
    f: F,
}

impl<T, U, F> EventRoutine for Map<T, F>
where
    T: EventRoutine,
    F: FnOnce(T::Return) -> U,
{
    type Return = U;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let Self { t, f } = self;
        match t.handle(data, view, event_or_peek) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => Handled::Return(f(r)),
        }
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        self.t.view(data, view, context, frame)
    }
}

enum AndThenPrivate<T, U, F> {
    First { t: T, f: F },
    Second(U),
}

pub struct AndThen<T, U, F>(AndThenPrivate<T, U, F>);

impl<T, U, F> EventRoutine for AndThen<T, U, F>
where
    T: EventRoutine,
    U: EventRoutine<Data = T::Data, View = T::View, Event = T::Event>,
    F: FnOnce(T::Return) -> U,
{
    type Return = U::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        match self.0 {
            AndThenPrivate::First { t, f } => match t.handle(data, view, event_or_peek) {
                Handled::Continue(t) => Handled::Continue(AndThen(AndThenPrivate::First { t, f })),
                Handled::Return(r) => f(r)
                    .handle(data, view, Peek::new())
                    .map_continue(|u| AndThen(AndThenPrivate::Second(u))),
            },
            AndThenPrivate::Second(u) => u
                .handle(data, view, event_or_peek)
                .map_continue(|u| AndThen(AndThenPrivate::Second(u))),
        }
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        match self.0 {
            AndThenPrivate::First { ref t, .. } => t.view(data, view, context, frame),
            AndThenPrivate::Second(ref u) => u.view(data, view, context, frame),
        }
    }
}

pub trait DataSelector {
    type DataInput;
    type DataOutput;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput;
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput;
}

pub trait ViewSelector {
    type ViewInput;
    type ViewOutput;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput;
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput;
}

pub trait Selector: DataSelector + ViewSelector {}

#[derive(Clone, Copy)]
pub struct Select<T, S> {
    t: T,
    selector: S,
}

impl<T, S> EventRoutine for Select<T, S>
where
    T: EventRoutine,
    S: Selector<DataOutput = T::Data, ViewOutput = T::View>,
{
    type Return = T::Return;
    type Data = S::DataInput;
    type View = S::ViewInput;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let Self { t, selector } = self;
        t.handle(selector.data_mut(data), selector.view(view), event_or_peek)
            .map_continue(|t| Self { t, selector })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        self.t
            .view(self.selector.data(data), self.selector.view_mut(view), context, frame)
    }
}

pub struct Repeat<T, F> {
    t: T,
    f: F,
}
impl<T, U, F> EventRoutine for Repeat<T, F>
where
    T: EventRoutine,
    F: FnMut(T::Return) -> Handled<U, T>,
{
    type Return = U;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let Self { t, mut f } = self;
        match t.handle(data, view, event_or_peek) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => match f(r) {
                Handled::Continue(t) => Handled::Continue(Self { t, f }),
                Handled::Return(r) => Handled::Return(r),
            },
        }
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        self.t.view(data, view, context, frame)
    }
}

pub struct EventRoutineView<'e, 'v, E: EventRoutine> {
    pub event_routine: &'e E,
    pub view: &'v mut E::View,
}

impl<'e, 'v, 'd, E> View<&'d E::Data> for EventRoutineView<'e, 'v, E>
where
    E: EventRoutine,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: &'d E::Data, context: ViewContext<C>, frame: &mut F) {
        self.event_routine.view(data, self.view, context, frame)
    }
}

pub trait Decorate {
    type View;
    type Data;

    fn view<E, F, C>(
        data: &Self::Data,
        event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify;
}

pub struct Decorated<T, D> {
    t: T,
    d: D,
}

impl<T, D> EventRoutine for Decorated<T, D>
where
    T: EventRoutine,
    D: Decorate<View = T::View, Data = T::Data>,
{
    type Return = T::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let Self { t, d } = self;
        t.handle(data, view, event_or_peek).map_continue(|t| Self { t, d })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        let event_routine_view = EventRoutineView {
            event_routine: &self.t,
            view,
        };
        D::view(data, event_routine_view, context, frame)
    }
}

pub struct OnEvent<T, F> {
    t: T,
    f: F,
}

impl<T, F> EventRoutine for OnEvent<T, F>
where
    T: EventRoutine,
    F: FnMut(&mut &mut T::Data, &T::Event),
{
    type Return = T::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let on_peek = |(s, data)| {
            let Self { t, f } = s;
            t.handle(data, view, Peek::new()).map_continue(|t| Self { t, f })
        };
        let on_event = |(s, mut data), event| {
            let Self { t, mut f } = s;
            (f)(&mut data, &event);
            t.handle(data, view, Event::new(event)).map_continue(|t| Self { t, f })
        };
        event_or_peek.with((self, data), on_event, on_peek)
    }

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        self.t.view(data, view, context, frame);
    }
}

make_either!(Either = Left | Right);

#[macro_export]
macro_rules! make_either {
    ($type:ident = $first:ident | $($rest:ident)|*) => {
        pub enum $type<$first, $($rest),*> {
            $first($first),
            $($rest($rest)),*
        }
        impl<$first, $($rest),*> EventRoutine for $type<$first, $($rest),*>
            where
                $first: EventRoutine,
                $($rest: EventRoutine<Data = $first::Data, View = $first::View, Return = $first::Return, Event = $first::Event>),*
        {
            type Return = $first::Return;
            type Data = $first::Data;
            type View = $first::View;
            type Event = $first::Event;

            fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
            where
                EP: EventOrPeek<Event = Self::Event>,
            {
                match self {
                    $type::$first(x) => x.handle(data, view, event_or_peek).map_continue($type::$first),
                    $($type::$rest(x) => x.handle(data, view, event_or_peek).map_continue($type::$rest)),*
                }
            }
            fn view<FR, CM>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<CM>, frame: &mut FR)
            where
                FR: Frame,
                CM: ColModify,
            {
                match self {
                    $type::$first(x) => x.view(data, view, context, frame),
                    $($type::$rest(x) => x.view(data, view, context, frame)),*
                }
            }
        }
    };
}
