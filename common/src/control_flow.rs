use crate::{
    align::{Align, Alignment},
    border::{Border, BorderStyle},
    fill::Fill,
    pad_to::PadTo,
};
use chargrid_core::{app, ctx_tint, input, Component, Ctx, Event, FrameBuffer, Rgba32, Size, Tint};
use std::marker::PhantomData;
use std::time::Duration;

pub struct CF<C: Component>(C);
pub fn cf<C: Component>(component: C) -> CF<C> {
    CF(component)
}

impl<C: Component> Component for CF<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.0.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

impl<C: Component> CF<C> {
    pub fn overlay<D: Component<State = C::State>, T: Tint>(
        self,
        background: D,
        tint: T,
        depth_delta: i8,
    ) -> CF<Overlay<C, D, T>> {
        cf(Overlay {
            foreground: self.0,
            background,
            tint,
            depth_delta,
        })
    }

    pub fn lens_state<S, L>(self, lens: L) -> CF<LensState<S, L, C>>
    where
        L: Lens<Input = S, Output = C::State>,
    {
        cf(LensState {
            state: PhantomData,
            lens,
            component: self.0,
        })
    }

    pub fn fill(self, background: Rgba32) -> CF<Fill<C>> {
        cf(Fill {
            component: self.0,
            background,
        })
    }

    pub fn border(self, style: BorderStyle) -> CF<Border<C>> {
        cf(Border {
            component: self.0,
            style,
        })
    }

    pub fn pad_to(self, size: Size) -> CF<PadTo<C>> {
        cf(PadTo {
            component: self.0,
            size,
        })
    }

    pub fn align(self, alignment: Alignment) -> CF<Align<C>> {
        cf(Align {
            component: self.0,
            alignment,
        })
    }

    pub fn centre(self) -> CF<Align<C>> {
        self.align(Alignment::centre())
    }
}

impl<S: Sized, C: Component<State = S>> CF<C> {
    pub fn with_state(self, state: C::State) -> CF<WithState<C>> {
        cf(WithState {
            component: self.0,
            state,
        })
    }
}

impl<C: Component<Output = ()>> CF<C> {
    pub fn delay(self, duration: Duration) -> CF<Delay<C>> {
        cf(Delay {
            component: self.0,
            remaining: duration,
        })
    }
    pub fn press_any_key(self) -> CF<PressAnyKey<C>> {
        cf(PressAnyKey(self.0))
    }
}

impl<T, C: Component<Output = Option<T>>> CF<C> {
    pub fn and_then<U, D, F>(self, f: F) -> CF<AndThen<C, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce(T) -> D,
    {
        cf(AndThen::First {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn map<U, F>(self, f: F) -> CF<Map<C, F>>
    where
        F: FnMut(T) -> U,
    {
        cf(Map {
            component: self.0,
            f,
        })
    }

    pub fn clear_each_frame(self) -> CF<ClearEachFrame<C>> {
        cf(ClearEachFrame(self.0))
    }

    pub fn catch_escape(self) -> CF<CatchEscape<C>> {
        cf(CatchEscape(self.0))
    }

    pub fn continue_<Br>(self) -> CF<Map<C, impl FnMut(T) -> LoopControl<T, Br>>> {
        self.map(LoopControl::Continue)
    }

    pub fn break_<Co>(self) -> CF<Map<C, impl FnMut(T) -> LoopControl<Co, T>>> {
        self.map(LoopControl::Break)
    }
}

impl<C: Component<State = ()>> CF<C> {
    pub fn ignore_state<S>(self) -> CF<IgnoreState<S, C>> {
        cf(IgnoreState {
            state: PhantomData,
            component: self.0,
        })
    }
}

impl<C: Component<Output = app::Output>> CF<C> {
    pub fn exit_on_close(self) -> CF<ExitOnClose<C>> {
        cf(ExitOnClose(self.0))
    }
}

pub struct Val<T: Clone>(pub T);
pub fn val<S, T: Clone>(t: T) -> CF<IgnoreState<S, Val<T>>> {
    cf(Val(t)).ignore_state()
}
impl<T: Clone> Component for Val<T> {
    type Output = Option<T>;
    type State = ();
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        Some(self.0.clone())
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(0, 0)
    }
}

pub struct ValOnce<T>(Option<T>);
impl<T> ValOnce<T> {
    pub fn new(t: T) -> Self {
        Self(Some(t))
    }
}
pub fn val_once<S, T>(t: T) -> CF<IgnoreState<S, ValOnce<T>>> {
    cf(ValOnce::new(t)).ignore_state()
}
impl<T> Component for ValOnce<T> {
    type Output = Option<T>;
    type State = ();
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        self.0.take()
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(0, 0)
    }
}

pub struct WithState<C: Component> {
    component: C,
    state: C::State,
}
impl<C> Component for WithState<C>
where
    C: Component,
{
    type Output = C::Output;
    type State = ();
    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&self.state, ctx, fb);
    }
    fn update(&mut self, _state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(&mut self.state, ctx, event)
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(&self.state, ctx)
    }
}

#[derive(Clone, Copy)]
pub enum LoopControl<Co, Br> {
    Continue(Co),
    Break(Br),
}
pub struct LoopState<S, C, F> {
    state: S,
    component: C,
    f: F,
}
pub fn loop_state<S, Co, Br, C, F>(state: S, init: Co, mut f: F) -> CF<LoopState<S, C, F>>
where
    C: Component<Output = Option<LoopControl<Co, Br>>, State = S>,
    F: FnMut(Co) -> C,
{
    cf(LoopState {
        component: f(init),
        state,
        f,
    })
}
impl<S, Co, Br, C, F> Component for LoopState<S, C, F>
where
    C: Component<Output = Option<LoopControl<Co, Br>>, State = S>,
    F: FnMut(Co) -> C,
{
    type Output = Option<Br>;
    type State = ();
    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&self.state, ctx, fb);
    }
    fn update(&mut self, _state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Some(control) = self.component.update(&mut self.state, ctx, event) {
            match control {
                LoopControl::Continue(co) => {
                    self.component = (self.f)(co);
                    None
                }
                LoopControl::Break(br) => Some(br),
            }
        } else {
            None
        }
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(&self.state, ctx)
    }
}
pub struct Loop<C, F> {
    component: C,
    f: F,
}
pub fn loop_<Co, Br, C, F>(init: Co, mut f: F) -> CF<Loop<C, F>>
where
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co) -> C,
{
    cf(Loop {
        component: f(init),
        f,
    })
}
impl<Co, Br, C, F> Component for Loop<C, F>
where
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co) -> C,
{
    type Output = Option<Br>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Some(control) = self.component.update(state, ctx, event) {
            match control {
                LoopControl::Continue(co) => {
                    self.component = (self.f)(co);
                    None
                }
                LoopControl::Break(br) => Some(br),
            }
        } else {
            None
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct OnState<S, T, F> {
    state: PhantomData<S>,
    output: PhantomData<T>,
    f: Option<F>,
}
pub fn on_state<S, T, F>(f: F) -> CF<OnState<S, T, F>>
where
    F: FnOnce(&mut S) -> T,
{
    cf(OnState {
        state: PhantomData,
        output: PhantomData,
        f: Some(f),
    })
}
impl<S, T, F> Component for OnState<S, T, F>
where
    F: FnOnce(&mut S) -> T,
{
    type Output = Option<T>;
    type State = S;

    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {
        panic!("this component should not live long enough to be rendered");
    }
    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        Some((self
            .f
            .take()
            .expect("this component should only be updated once"))(
            state
        ))
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        panic!("nothing should be checking the size of this component")
    }
}

pub enum OnStateThen<C: Component, F> {
    Component(C),
    F(Option<F>),
}
pub fn on_state_then<C, F>(f: F) -> CF<OnStateThen<C, F>>
where
    C: Component,
    F: FnOnce(&mut C::State) -> C,
{
    cf(OnStateThen::F(Some(f)))
}
impl<C, F> Component for OnStateThen<C, F>
where
    C: Component,
    F: FnOnce(&mut C::State) -> C,
{
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        match self {
            Self::F(_) => panic!("state F should not last long enough to get here"),
            Self::Component(component) => component.render(state, ctx, fb),
        }
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self {
            Self::F(f) => {
                let mut component = (f.take().unwrap())(state);
                let result = component.update(state, ctx, event);
                *self = Self::Component(component);
                result
            }
            Self::Component(component) => component.update(state, ctx, event),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        match self {
            Self::F(_) => panic!("state F should not last long enough to get here"),
            Self::Component(component) => component.size(state, ctx),
        }
    }
}

pub struct Render<F: Fn(Ctx, &mut FrameBuffer)> {
    f: F,
}
pub fn render<F: Fn(Ctx, &mut FrameBuffer)>(f: F) -> CF<Render<F>> {
    cf(Render { f })
}
impl<F> Component for Render<F>
where
    F: Fn(Ctx, &mut FrameBuffer),
{
    type Output = ();
    type State = ();
    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        (self.f)(ctx, fb);
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {}
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

pub struct IgnoreState<S, C: Component<State = ()>> {
    state: PhantomData<S>,
    component: C,
}

impl<S, C> Component for IgnoreState<S, C>
where
    C: Component<State = ()>,
{
    type Output = C::Output;
    type State = S;

    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&(), ctx, fb);
    }
    fn update(&mut self, _state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(&mut (), ctx, event)
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(&(), ctx)
    }
}

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

pub enum AndThen<C, D, F> {
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First { component: C, f: Option<F> },
    Second(D),
}

impl<T, U, C, D, F> Component for AndThen<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce(T) -> D,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        match self {
            Self::First { component, .. } => component.render(state, ctx, fb),
            Self::Second(component) => component.render(state, ctx, fb),
        }
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self {
            Self::First {
                component,
                ref mut f,
            } => match component.update(state, ctx, event) {
                None => None,
                Some(t) => {
                    let mut d = (f.take().unwrap())(t);
                    let peek_result = d.update(state, ctx, Event::Peek);
                    *self = Self::Second(d);
                    peek_result
                }
            },
            Self::Second(component) => component.update(state, ctx, event),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        match self {
            Self::First { component, .. } => component.size(state, ctx),
            Self::Second(component) => component.size(state, ctx),
        }
    }
}

pub struct Map<C, F> {
    component: C,
    f: F,
}

impl<T, U, C, F> Component for Map<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnMut(T) -> U,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self.component.update(state, ctx, event) {
            None => None,
            Some(t) => Some((self.f)(t)),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct ClearEachFrame<C: Component>(pub C);

impl<C> Component for ClearEachFrame<C>
where
    C: Component,
{
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        fb.clear();
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.0.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct ExitOnClose<C: Component<Output = app::Output>>(pub C);

impl<C> Component for ExitOnClose<C>
where
    C: Component<Output = app::Output>,
{
    type Output = app::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Input(input::Input::Keyboard(input::keys::ETX)) = event {
            return Some(app::Exit);
        }
        self.0.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Escape;
pub type OrEscape<T> = Result<T, Escape>;

pub struct CatchEscape<C: Component>(pub C);

impl<T, C> Component for CatchEscape<C>
where
    C: Component<Output = Option<T>>,
{
    type Output = Option<OrEscape<T>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) = event {
            return Some(Err(Escape));
        }
        self.0.update(state, ctx, event).map(Ok)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct Delay<C: Component<Output = ()>> {
    component: C,
    remaining: Duration,
}
impl<C> Component for Delay<C>
where
    C: Component<Output = ()>,
{
    type Output = Option<()>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(state, ctx, event);
        if let Event::Tick(duration) = event {
            if let Some(remaining) = self.remaining.checked_sub(duration) {
                self.remaining = remaining;
            } else {
                self.remaining = Duration::from_millis(0);
                return Some(());
            }
        }
        None
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct PressAnyKey<C: Component<Output = ()>>(C);
impl<C> Component for PressAnyKey<C>
where
    C: Component<Output = ()>,
{
    type Output = Option<()>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.0.update(state, ctx, event);
        if let Event::Input(input::Input::Keyboard(_)) = event {
            Some(())
        } else {
            None
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct Overlay<C: Component, D: Component, T: Tint> {
    pub foreground: C,
    pub background: D,
    pub tint: T,
    pub depth_delta: i8,
}

impl<C, D, T> Component for Overlay<C, D, T>
where
    C: Component,
    D: Component<State = C::State>,
    T: Tint,
{
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.background.render(
            state,
            ctx_tint!(ctx, self.tint).add_depth(-self.depth_delta),
            fb,
        );
        self.foreground.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.foreground.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.foreground.size(state, ctx)
    }
}

#[macro_export]
macro_rules! either {
    ($type:ident = $first:ident | $($rest:ident)|*) => {
        pub enum $type<$first, $($rest),*> {
            $first($first),
            $($rest($rest)),*
        }
        impl<$first, $($rest),*> Component for $type<$first, $($rest),*>
            where
                $first: Component,
                $($rest: Component<Output = $first::Output, State = $first::State>),*
        {
            type Output = $first::Output;
            type State = $first::State;

            fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
                match self {
                    $type::$first(x) => x.render(state, ctx, fb),
                    $($type::$rest(x) => x.render(state, ctx, fb)),*
                }
            }
            fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
                match self {
                    $type::$first(x) => x.update(state, ctx, event),
                    $($type::$rest(x) => x.update(state, ctx, event)),*
                }
            }
            fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
                match self {
                    $type::$first(x) => x.size(state, ctx),
                    $($type::$rest(x) => x.size(state, ctx)),*
                }
            }
        }
    };
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

pub use crate::{either, lens};
