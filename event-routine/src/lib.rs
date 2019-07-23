pub extern crate prototty_input;
pub extern crate prototty_render;

use prototty_input::Input;
use prototty_render::{Frame, ViewContext, ViewTransformRgb24};
use std::marker::PhantomData;
use std::time::Duration;

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

pub trait EventRoutine: Sized {
    type Return;
    type Data;
    type View;
    type Event;

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self>;

    fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24;

    fn peek(self, _data: &mut Self::Data, _view: &Self::View) -> Handled<Self::Return, Self> {
        Handled::Continue(self)
    }

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
        AndThen::First { t: self, f }
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Return) -> U,
    {
        Map { t: self, f }
    }

    fn map_side_effect<F, U>(self, f: F) -> MapSideEffect<Self, F>
    where
        F: FnOnce(Self::Return, &mut Self::Data, &Self::View) -> U,
    {
        MapSideEffect { t: self, f }
    }

    fn convert_input_to_common_event(self) -> ConvertInputToCommonEvent<Self> {
        ConvertInputToCommonEvent(self)
    }
}

pub struct Value<T, D, V, E> {
    value: T,
    data: PhantomData<D>,
    view: PhantomData<V>,
    event: PhantomData<E>,
}

impl<T, D, V, E> Value<T, D, V, E> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            data: PhantomData,
            view: PhantomData,
            event: PhantomData,
        }
    }
}

impl<T, D, V, E> EventRoutine for Value<T, D, V, E> {
    type Return = T;
    type Data = D;
    type View = V;
    type Event = E;

    fn handle_event(
        self,
        data: &mut Self::Data,
        view: &Self::View,
        _event: Self::Event,
    ) -> Handled<Self::Return, Self> {
        self.peek(data, view)
    }

    fn view<F, R>(&self, _data: &Self::Data, _view: &mut Self::View, _context: ViewContext<R>, _frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
    }

    fn peek(self, _data: &mut Self::Data, _view: &Self::View) -> Handled<Self::Return, Self> {
        Handled::Return(self.value)
    }
}

pub struct MapSideEffect<T, F> {
    t: T,
    f: F,
}
impl<T, U, F> EventRoutine for MapSideEffect<T, F>
where
    T: EventRoutine,
    F: FnOnce(T::Return, &mut T::Data, &T::View) -> U,
{
    type Return = U;
    type Data = T::Data;
    type View = T::View;
    type Event = T::Event;

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        let Self { t, f } = self;
        match t.handle_event(data, view, event) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => Handled::Return(f(r, data, view)),
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        self.t.view(data, view, context, frame)
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        let Self { t, f } = self;
        match t.peek(data, view) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => Handled::Return(f(r, data, view)),
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

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        let Self { t, f } = self;
        match t.handle_event(data, view, event) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => Handled::Return(f(r)),
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        self.t.view(data, view, context, frame)
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        let Self { t, f } = self;
        match t.peek(data, view) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => Handled::Return(f(r)),
        }
    }
}

pub enum AndThen<T, U, F> {
    First { t: T, f: F },
    Second(U),
}

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

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        match self {
            AndThen::First { t, f } => match t.handle_event(data, view, event) {
                Handled::Continue(t) => Handled::Continue(AndThen::First { t, f }),
                Handled::Return(r) => f(r).peek(data, view).map_continue(AndThen::Second),
            },
            AndThen::Second(u) => u.handle_event(data, view, event).map_continue(AndThen::Second),
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        match self {
            AndThen::First { ref t, .. } => t.view(data, view, context, frame),
            AndThen::Second(ref u) => u.view(data, view, context, frame),
        }
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        match self {
            AndThen::First { t, f } => match t.peek(data, view) {
                Handled::Continue(t) => Handled::Continue(AndThen::First { t, f }),
                Handled::Return(r) => f(r).peek(data, view).map_continue(AndThen::Second),
            },
            AndThen::Second(u) => u.peek(data, view).map_continue(AndThen::Second),
        }
    }
}

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> EventRoutine for Either<A, B>
where
    A: EventRoutine,
    B: EventRoutine<Data = A::Data, View = A::View, Return = A::Return, Event = A::Event>,
{
    type Return = A::Return;
    type Data = A::Data;
    type View = A::View;
    type Event = A::Event;

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        match self {
            Either::A(a) => a.handle_event(data, view, event).map_continue(Either::A),
            Either::B(b) => b.handle_event(data, view, event).map_continue(Either::B),
        }
    }

    fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
        match self {
            Either::A(a) => a.view(data, view, context, frame),
            Either::B(b) => b.view(data, view, context, frame),
        }
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        match self {
            Either::A(a) => a.peek(data, view).map_continue(Either::A),
            Either::B(b) => b.peek(data, view).map_continue(Either::B),
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

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        let Self { t, selector } = self;
        t.handle_event(selector.data_mut(data), selector.view(view), event)
            .map_continue(|t| Self { t, selector })
    }

    fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
        self.t
            .view(self.selector.data(data), self.selector.view_mut(view), context, frame)
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        let Self { t, selector } = self;
        t.peek(selector.data_mut(data), selector.view(view))
            .map_continue(|t| Self { t, selector })
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

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        let Self { t, mut f } = self;
        match t.handle_event(data, view, event) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => match f(r) {
                Handled::Continue(t) => Handled::Continue(Self { t, f }),
                Handled::Return(r) => Handled::Return(r),
            },
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        self.t.view(data, view, context, frame)
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        let Self { t, mut f } = self;
        match t.peek(data, view) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => match f(r) {
                Handled::Continue(t) => Handled::Continue(Self { t, f }),
                Handled::Return(r) => Handled::Return(r),
            },
        }
    }
}

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

pub struct ConvertInputToCommonEvent<T>(T);

impl<T> EventRoutine for ConvertInputToCommonEvent<T>
where
    T: EventRoutine<Event = Input>,
{
    type Return = T::Return;
    type Data = T::Data;
    type View = T::View;
    type Event = CommonEvent;

    fn handle_event(self, data: &mut Self::Data, view: &Self::View, event: Self::Event) -> Handled<Self::Return, Self> {
        if let CommonEvent::Input(input) = event {
            match self.0.handle_event(data, view, input) {
                Handled::Continue(t) => Handled::Continue(Self(t)),
                Handled::Return(r) => Handled::Return(r),
            }
        } else {
            Handled::Continue(self)
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        self.0.view(data, view, context, frame)
    }

    fn peek(self, data: &mut Self::Data, view: &Self::View) -> Handled<Self::Return, Self> {
        self.0.peek(data, view).map_continue(Self)
    }
}
