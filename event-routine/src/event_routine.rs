use crate::Handled;
use prototty_input::Input;
use prototty_render::{Frame, ViewContext, ViewTransformRgb24};
use std::marker::PhantomData;
use std::time::Duration;

pub trait EventRoutine: Sized {
    type Return;
    type Data;
    type View;
    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>;

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
    fn handle<I>(
        self,
        data: &mut Self::Data,
        _inputs: I,
        _view: &Self::View,
        _duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        let Self { t, f } = self;
        match t.handle(data, inputs, view, duration) {
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        let Self { t, f } = self;
        match t.handle(data, inputs, view, duration) {
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        match self {
            AndThen::First { t, f } => match t.handle(data, inputs, view, duration) {
                Handled::Continue(t) => Handled::Continue(AndThen::First { t, f }),
                Handled::Return(r) => f(r).peek(data).map_continue(AndThen::Second),
            },
            AndThen::Second(u) => u.handle(data, inputs, view, duration).map_continue(AndThen::Second),
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        match self {
            Either::A(a) => a.handle(data, inputs, view, duration).map_continue(Either::A),
            Either::B(b) => b.handle(data, inputs, view, duration).map_continue(Either::B),
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        let Self { t, selector } = self;
        t.handle(selector.data_mut(data), inputs, selector.view(view), duration)
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

    fn handle<I>(
        self,
        data: &mut Self::Data,
        inputs: I,
        view: &Self::View,
        duration: Duration,
    ) -> Handled<Self::Return, Self>
    where
        I: Iterator<Item = Input>,
    {
        let Self { t, mut f } = self;
        match t.handle(data, inputs, view, duration) {
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
