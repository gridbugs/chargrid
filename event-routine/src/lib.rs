pub extern crate prototty_input;
pub extern crate prototty_render;

use prototty_render::{Frame, ViewContext, ViewTransformRgb24};
use std::marker::PhantomData;

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

pub mod event {
    use prototty_input::Input as ProtottyInput;
    use std::time::Duration;

    pub struct Input(pub ProtottyInput);

    pub struct Frame(pub Duration);

    pub enum Dynamic {
        Input(ProtottyInput),
        Frame(Duration),
    }

    pub trait Trait: private::Sealed {
        fn dynamic(self) -> Dynamic;
        fn input(self) -> Option<ProtottyInput>;
        fn frame(self) -> Option<Duration>;
    }

    impl Trait for Input {
        fn dynamic(self) -> Dynamic {
            Dynamic::Input(self.0)
        }
        fn input(self) -> Option<ProtottyInput> {
            Some(self.0)
        }
        fn frame(self) -> Option<Duration> {
            None
        }
    }

    impl Trait for Frame {
        fn dynamic(self) -> Dynamic {
            Dynamic::Frame(self.0)
        }
        fn input(self) -> Option<ProtottyInput> {
            None
        }
        fn frame(self) -> Option<Duration> {
            Some(self.0)
        }
    }

    mod private {
        pub trait Sealed {}

        impl Sealed for super::Input {}
        impl Sealed for super::Frame {}
    }
}

pub use event::Trait as Event;

pub trait EventRoutine: Sized {
    type Return;
    type Data;
    type View;

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event;

    fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24;

    fn peek(self, _data: &mut Self::Data) -> Handled<Self::Return, Self> {
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
}

pub struct Value<T, D, V> {
    value: T,
    data: PhantomData<D>,
    view: PhantomData<V>,
}

impl<T, D, V> Value<T, D, V> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            data: PhantomData,
            view: PhantomData,
        }
    }
}

impl<T, D, V> EventRoutine for Value<T, D, V> {
    type Return = T;
    type Data = D;
    type View = V;

    fn handle_event<E>(self, data: &mut Self::Data, _view: &Self::View, _event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
        self.peek(data)
    }

    fn view<F, R>(&self, _data: &Self::Data, _view: &mut Self::View, _context: ViewContext<R>, _frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
    }

    fn peek(self, _data: &mut Self::Data) -> Handled<Self::Return, Self> {
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

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
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

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
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
}

pub enum AndThen<T, U, F> {
    First { t: T, f: F },
    Second(U),
}

impl<T, U, F> EventRoutine for AndThen<T, U, F>
where
    T: EventRoutine,
    U: EventRoutine<Data = T::Data, View = T::View>,
    F: FnOnce(T::Return) -> U,
{
    type Return = U::Return;
    type Data = T::Data;
    type View = T::View;

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
        match self {
            AndThen::First { t, f } => match t.handle_event(data, view, event) {
                Handled::Continue(t) => Handled::Continue(AndThen::First { t, f }),
                Handled::Return(r) => f(r).peek(data).map_continue(AndThen::Second),
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
}

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> EventRoutine for Either<A, B>
where
    A: EventRoutine,
    B: EventRoutine<Data = A::Data, View = A::View, Return = A::Return>,
{
    type Return = A::Return;
    type Data = A::Data;
    type View = A::View;

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
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

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
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

    fn handle_event<E>(self, data: &mut Self::Data, view: &Self::View, event: E) -> Handled<Self::Return, Self>
    where
        E: Event,
    {
        let Self { t, mut f } = self;
        match t.handle_event(data, view, event) {
            Handled::Continue(t) => Handled::Continue(Self { t, f }),
            Handled::Return(r) => f(r).map_continue(|t| Self { t, f }),
        }
    }

    fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<R>, frame: &mut G)
    where
        G: Frame,
        R: ViewTransformRgb24,
    {
        self.t.view(data, view, context, frame)
    }
}
