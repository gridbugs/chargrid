use chargrid_core::{Component, Ctx, Event, FrameBuffer, Size};
use std::marker::PhantomData;

pub mod boxed;
pub mod unboxed;

pub trait Lens {
    type Input;
    type Output;
    fn get<'a>(&self, input: &'a Self::Input) -> &'a Self::Output;
    fn get_mut<'a>(&mut self, input: &'a mut Self::Input) -> &'a mut Self::Output;
}

pub struct LensFns<I, O, R, M> {
    input: PhantomData<I>,
    output: PhantomData<O>,
    get: R,
    get_mut: M,
}

impl<I, O, R, M> LensFns<I, O, R, M> {
    pub fn new(get: R, get_mut: M) -> Self {
        Self {
            input: PhantomData,
            output: PhantomData,
            get,
            get_mut,
        }
    }
}

impl<I, O, R, M> Lens for LensFns<I, O, R, M>
where
    R: Fn(&I) -> &O,
    M: FnMut(&mut I) -> &mut O,
{
    type Input = I;
    type Output = O;
    fn get<'a>(&self, input: &'a Self::Input) -> &'a Self::Output {
        (self.get)(input)
    }
    fn get_mut<'a>(&mut self, input: &'a mut Self::Input) -> &'a mut Self::Output {
        (self.get_mut)(input)
    }
}

pub struct LensState<S, L, C>
where
    C: Component,
    L: Lens<Input = S, Output = C::State>,
{
    state: PhantomData<S>,
    lens: L,
    component: C,
}

impl<S, L, C> Component for LensState<S, L, C>
where
    C: Component,
    L: Lens<Input = S, Output = C::State>,
{
    type Output = C::Output;
    type State = S;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&self.lens.get(state), ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(self.lens.get_mut(state), ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(self.lens.get(state), ctx)
    }
}

#[macro_export]
macro_rules! lens {
    ($input:ty[$field:ident]: $output:ty) => {{
        fn get(state: &$input) -> &$output {
            &state.$field
        }
        fn get_mut(state: &mut $input) -> &mut $output {
            &mut state.$field
        }
        LensFns::new(get, get_mut)
    }};
}

#[derive(Clone, Copy)]
pub enum LoopControl<Co, Br> {
    Continue(Co),
    Break(Br),
}

impl<Co, Br> LoopControl<Co, Br> {
    pub fn map_continue<Co2, F: FnOnce(Co) -> Co2>(self, f: F) -> LoopControl<Co2, Br> {
        match self {
            Self::Continue(co) => LoopControl::Continue(f(co)),
            Self::Break(br) => LoopControl::Break(br),
        }
    }
    pub fn map_break<Br2, F: FnOnce(Br) -> Br2>(self, f: F) -> LoopControl<Co, Br2> {
        match self {
            Self::Continue(co) => LoopControl::Continue(co),
            Self::Break(br) => LoopControl::Break(f(br)),
        }
    }
}

pub struct Close;
pub type OrClose<T> = Result<T, Close>;

#[derive(Clone, Copy, Debug)]
pub struct Escape;
pub type OrEscape<T> = Result<T, Escape>;

pub enum EscapeOrStart {
    Escape,
    Start,
}

pub type OrEscapeOrStart<T> = Result<T, EscapeOrStart>;

pub use crate::lens;
