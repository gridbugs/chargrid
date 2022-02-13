use crate::{
    add_offset::AddOffset,
    align::{Align, Alignment},
    border::{Border, BorderStyle},
    bound_size::{BoundHeight, BoundSize, BoundWidth},
    fill::Fill,
    pad_by::{PadBy, Padding},
    pad_to::PadTo,
    set_size::{SetHeight, SetSize, SetWidth},
    text::StyledString,
};
use chargrid_core::{
    app, ctx_tint, input, BoxedComponent, Component, Coord, Ctx, Event, FrameBuffer, Rgba32, Size,
    Style, Tint, TintIdentity,
};
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

    pub fn some(self) -> CF<Some_<C>> {
        cf(Some_(self.0))
    }

    pub fn none(self) -> CF<None_<C>> {
        cf(None_(self.0))
    }

    pub fn clear_each_frame(self) -> CF<ClearEachFrame<C>> {
        cf(ClearEachFrame(self.0))
    }

    pub fn overlay_tint<D: Component<State = C::State>, T: Tint>(
        self,
        background: D,
        tint: T,
        depth_delta: i8,
    ) -> CF<OverlayTint<C, D, T>> {
        cf(OverlayTint {
            foreground: self.0,
            background,
            tint,
            depth_delta,
        })
    }

    pub fn overlay<D: Component<State = C::State>>(
        self,
        background: D,
        depth_delta: i8,
    ) -> CF<OverlayTint<C, D, TintIdentity>> {
        cf(OverlayTint {
            foreground: self.0,
            background,
            tint: TintIdentity,
            depth_delta,
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

    pub fn pad_by(self, padding: Padding) -> CF<PadBy<C>> {
        cf(PadBy {
            component: self.0,
            padding,
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

    pub fn add_offset(self, offset: Coord) -> CF<AddOffset<C>> {
        cf(AddOffset {
            component: self.0,
            offset,
        })
    }

    pub fn add_x(self, x: i32) -> CF<AddOffset<C>> {
        cf(AddOffset {
            component: self.0,
            offset: Coord { x, y: 0 },
        })
    }

    pub fn add_y(self, y: i32) -> CF<AddOffset<C>> {
        cf(AddOffset {
            component: self.0,
            offset: Coord { x: 0, y },
        })
    }

    pub fn set_size(self, size: Size) -> CF<SetSize<C>> {
        cf(SetSize {
            component: self.0,
            size,
        })
    }

    pub fn set_width(self, width: u32) -> CF<SetWidth<C>> {
        cf(SetWidth {
            component: self.0,
            width,
        })
    }

    pub fn set_height(self, height: u32) -> CF<SetHeight<C>> {
        cf(SetHeight {
            component: self.0,
            height,
        })
    }

    pub fn bound_size(self, size: Size) -> CF<BoundSize<C>> {
        cf(BoundSize {
            component: self.0,
            size,
        })
    }

    pub fn bound_width(self, width: u32) -> CF<BoundWidth<C>> {
        cf(BoundWidth {
            component: self.0,
            width,
        })
    }

    pub fn bound_height(self, height: u32) -> CF<BoundHeight<C>> {
        cf(BoundHeight {
            component: self.0,
            height,
        })
    }

    pub fn with_title<T: Component<State = C::State>>(
        self,
        title: T,
        padding: i32,
    ) -> CF<WithTitle<T, C>> {
        cf(WithTitle {
            title,
            component: self.0,
            padding,
        })
    }
}

impl<C: Component> CF<C>
where
    C::State: Sized,
{
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
    pub fn and_then_persistent<U, D, F>(self, f: F) -> CF<AndThenPersistent<C, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce(C, T) -> D,
    {
        cf(AndThenPersistent(AndThenPersistentPriv::First(Some(
            AndThenPersistentFirst {
                component: self.0,
                f,
            },
        ))))
    }

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

    pub fn then<U, D, F>(self, f: F) -> CF<Then<C, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce() -> D,
    {
        cf(Then::First {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn then_side_effect<F>(self, f: F) -> CF<ThenSideEffect<C, F>>
    where
        F: FnOnce(&mut C::State),
    {
        cf(ThenSideEffect {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn map<U, F>(self, f: F) -> CF<Map<C, F>>
    where
        F: FnOnce(T) -> U,
    {
        cf(Map {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn map_val<U, F>(self, f: F) -> CF<MapVal<C, F>>
    where
        F: FnOnce() -> U,
    {
        cf(MapVal {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn catch_escape(self) -> CF<CatchEscape<C>> {
        cf(CatchEscape(self.0))
    }

    #[cfg(feature = "gamepad")]
    pub fn catch_escape_or_start(self) -> CF<CatchEscapeOrStart<C>> {
        cf(CatchEscapeOrStart(self.0))
    }

    pub fn continue_<Br>(self) -> CF<Continue<C, Br>> {
        cf(Continue {
            component: self.0,
            br: PhantomData,
        })
    }

    pub fn break_<Co>(self) -> CF<Break<C, Co>> {
        cf(Break {
            component: self.0,
            co: PhantomData,
        })
    }

    pub fn continue_with<Br, U>(self, value: U) -> CF<Continue<Replace<C, U>, Br>> {
        self.replace(value).continue_()
    }

    pub fn break_with<Co, U>(self, value: U) -> CF<Break<Replace<C, U>, Co>> {
        self.replace(value).break_()
    }

    pub fn replace<U>(self, value: U) -> CF<Replace<C, U>> {
        cf(Replace {
            component: self.0,
            value: Some(value),
        })
    }

    pub fn pause(self) -> CF<Pause<C>> {
        cf(Pause(self.0))
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

impl<C: 'static + Component> CF<C>
where
    C::State: Sized,
{
    pub fn boxed(self) -> CF<BoxedComponent<C::Output, C::State>> {
        cf(BoxedComponent(Box::new(self.0)))
    }

    pub fn boxed_cf(self) -> BoxedCF<C::Output, C::State> {
        BoxedCF(self.boxed())
    }
}

pub struct BoxedCF<O, S>(CF<BoxedComponent<O, S>>);
pub fn boxed_cf<C: 'static + Component>(component: C) -> BoxedCF<C::Output, C::State>
where
    C::State: Sized,
{
    cf(component).boxed_cf()
}

impl<O, S> From<BoxedComponent<O, S>> for BoxedCF<O, S> {
    fn from(boxed_component: BoxedComponent<O, S>) -> Self {
        BoxedCF(cf(boxed_component))
    }
}

impl<O, S> Component for BoxedCF<O, S> {
    type Output = O;
    type State = S;
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

impl<O: 'static, S: 'static> BoxedCF<O, S> {
    pub fn lens_state<S_: 'static, L: 'static>(self, lens: L) -> BoxedCF<O, S_>
    where
        L: Lens<Input = S_, Output = S>,
    {
        self.0.lens_state(lens).boxed_cf()
    }

    pub fn some(self) -> BoxedCF<Option<O>, S> {
        self.0.some().boxed_cf()
    }

    pub fn none(self) -> BoxedCF<Option<O>, S> {
        self.0.none().boxed_cf()
    }

    pub fn overlay_tint<D: 'static + Component<State = S>, T: 'static + Tint>(
        self,
        background: D,
        tint: T,
        depth_delta: i8,
    ) -> Self {
        self.0
            .overlay_tint(background, tint, depth_delta)
            .boxed_cf()
    }

    pub fn overlay<D: 'static + Component<State = S>>(
        self,
        background: D,
        depth_delta: i8,
    ) -> Self {
        self.0.overlay(background, depth_delta).boxed_cf()
    }

    pub fn clear_each_frame(self) -> Self {
        self.0.clear_each_frame().boxed_cf()
    }

    pub fn fill(self, background: Rgba32) -> Self {
        self.0.fill(background).boxed_cf()
    }

    pub fn border(self, style: BorderStyle) -> Self {
        self.0.border(style).boxed_cf()
    }

    pub fn pad_to(self, size: Size) -> Self {
        self.0.pad_to(size).boxed_cf()
    }

    pub fn pad_by(self, padding: Padding) -> Self {
        self.0.pad_by(padding).boxed_cf()
    }

    pub fn align(self, alignment: Alignment) -> Self {
        self.0.align(alignment).boxed_cf()
    }

    pub fn centre(self) -> Self {
        self.0.centre().boxed_cf()
    }

    pub fn add_offset(self, offset: Coord) -> Self {
        self.0.add_offset(offset).boxed_cf()
    }

    pub fn add_x(self, x: i32) -> Self {
        self.0.add_x(x).boxed_cf()
    }

    pub fn add_y(self, y: i32) -> Self {
        self.0.add_y(y).boxed_cf()
    }

    pub fn set_size(self, size: Size) -> Self {
        self.0.set_size(size).boxed_cf()
    }

    pub fn set_width(self, width: u32) -> Self {
        self.0.set_width(width).boxed_cf()
    }

    pub fn set_height(self, height: u32) -> Self {
        self.0.set_height(height).boxed_cf()
    }

    pub fn bound_size(self, size: Size) -> Self {
        self.0.bound_size(size).boxed_cf()
    }

    pub fn bound_width(self, width: u32) -> Self {
        self.0.bound_width(width).boxed_cf()
    }

    pub fn bound_height(self, height: u32) -> Self {
        self.0.bound_height(height).boxed_cf()
    }

    pub fn with_title<T: 'static + Component<State = S>>(self, title: T, padding: i32) -> Self {
        self.0.with_title(title, padding).boxed_cf()
    }
}

impl<O: 'static, S: 'static> BoxedCF<O, S> {
    pub fn with_state(self, state: S) -> BoxedCF<O, ()> {
        self.0.with_state(state).boxed_cf()
    }
}

impl<S: 'static> BoxedCF<(), S> {
    pub fn delay(self, duration: Duration) -> BoxedCF<Option<()>, S> {
        self.0.delay(duration).boxed_cf()
    }

    pub fn press_any_key(self) -> BoxedCF<Option<()>, S> {
        self.0.press_any_key().boxed_cf()
    }
}

impl<T: 'static, S: 'static> BoxedCF<Option<T>, S> {
    pub fn and_then_persistent<U, D: 'static, F: 'static>(self, f: F) -> BoxedCF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(Self, T) -> D,
    {
        self.0.and_then_persistent(|a, b| f(a.into(), b)).boxed_cf()
    }

    pub fn and_then<U, D: 'static, F: 'static>(self, f: F) -> BoxedCF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(T) -> D,
    {
        self.0.and_then(f).boxed_cf()
    }

    pub fn then<U, D: 'static, F: 'static>(self, f: F) -> BoxedCF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce() -> D,
    {
        self.0.then(f).boxed_cf()
    }

    pub fn then_side_effect<F: 'static>(self, f: F) -> BoxedCF<Option<T>, S>
    where
        F: FnOnce(&mut S),
    {
        self.0.then_side_effect(f).boxed_cf()
    }

    pub fn map<U, F: 'static>(self, f: F) -> BoxedCF<Option<U>, S>
    where
        F: FnOnce(T) -> U,
    {
        self.0.map(f).boxed_cf()
    }

    pub fn map_val<U, F: 'static>(self, f: F) -> BoxedCF<Option<U>, S>
    where
        F: FnOnce() -> U,
    {
        self.0.map_val(f).boxed_cf()
    }

    pub fn catch_escape(self) -> BoxedCF<Option<OrEscape<T>>, S> {
        self.0.catch_escape().boxed_cf()
    }

    #[cfg(feature = "gamepad")]
    pub fn catch_escape_or_start(self) -> BoxedCF<Option<OrEscapeOrStart<T>>, S> {
        self.0.catch_escape_or_start().boxed_cf()
    }

    pub fn replace<U: 'static>(self, value: U) -> BoxedCF<Option<U>, S> {
        self.0.replace(value).boxed_cf()
    }

    pub fn continue_<Br: 'static>(self) -> BoxedCF<Option<LoopControl<T, Br>>, S> {
        self.0.continue_().boxed_cf()
    }

    pub fn break_<Co: 'static>(self) -> BoxedCF<Option<LoopControl<Co, T>>, S> {
        self.0.break_().boxed_cf()
    }

    pub fn continue_with<Br: 'static, U: 'static>(
        self,
        value: U,
    ) -> BoxedCF<Option<LoopControl<U, Br>>, S> {
        self.0.continue_with(value).boxed_cf()
    }

    pub fn break_with<Co: 'static, U: 'static>(
        self,
        value: U,
    ) -> BoxedCF<Option<LoopControl<Co, U>>, S> {
        self.0.break_with(value).boxed_cf()
    }

    pub fn repeat<A: 'static, O: 'static, F: 'static>(self, init: A, f: F) -> BoxedCF<Option<O>, S>
    where
        F: FnMut(A, T) -> BoxedCF<Option<LoopControl<A, O>>, S>,
    {
        boxed::loop_((self, f, init), |(self_, mut f, acc)| {
            self_.and_then_persistent(|self_, entry| {
                f(acc, entry).map(|loop_control| loop_control.map_continue(|c| (self_, f, c)))
            })
        })
    }

    pub fn repeat_unit<O: 'static, F: 'static>(self, mut f: F) -> BoxedCF<Option<O>, S>
    where
        F: FnMut(T) -> BoxedCF<Option<LoopControl<(), O>>, S>,
    {
        self.repeat((), move |(), entry| f(entry))
    }

    pub fn pause(self) -> BoxedCF<Option<T>, S> {
        self.0.pause().boxed_cf()
    }
}

impl<O: 'static> BoxedCF<O, ()> {
    pub fn ignore_state<S: 'static>(self) -> BoxedCF<O, S> {
        self.0.ignore_state().boxed_cf()
    }
}

impl<S: 'static> BoxedCF<app::Output, S> {
    pub fn exit_on_close(self) -> Self {
        self.0.exit_on_close().boxed_cf()
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

pub struct Never<T> {
    t: PhantomData<T>,
}
impl<T> Never<T> {
    pub fn new() -> Self {
        Self { t: PhantomData }
    }
}
pub fn never<S, T>() -> CF<IgnoreState<S, Never<T>>> {
    cf(Never::new()).ignore_state()
}
impl<T> Component for Never<T> {
    type Output = Option<T>;
    type State = ();
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        None
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(0, 0)
    }
}

/// Component decorator that creates an environment with a given state for its child,
/// and presents a state of `()` to its parent
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

/// Component decorator intended for use within `loop_`, which wraps yielded values
/// in `LoopControl::Continue`
pub struct Continue<C: Component, Br> {
    component: C,
    br: PhantomData<Br>,
}
impl<C, Co, Br> Component for Continue<C, Br>
where
    C: Component<Output = Option<Co>>,
{
    type Output = Option<LoopControl<Co, Br>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, ctx, event)
            .map(LoopControl::Continue)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(&state, ctx)
    }
}

/// Component decorator intended for use within `loop_`, which wraps yielded values
/// in `LoopControl::Break`
pub struct Break<C: Component, Co> {
    component: C,
    co: PhantomData<Co>,
}
impl<C, Co, Br> Component for Break<C, Co>
where
    C: Component<Output = Option<Br>>,
{
    type Output = Option<LoopControl<Co, Br>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(&state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, ctx, event)
            .map(LoopControl::Break)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(&state, ctx)
    }
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

pub struct LoopUnit<C, F> {
    component: C,
    f: F,
}
pub fn loop_unit<Br, C, F>(mut f: F) -> CF<LoopUnit<C, F>>
where
    C: Component<Output = Option<LoopControl<(), Br>>>,
    F: FnMut() -> C,
{
    cf(LoopUnit { component: f(), f })
}
impl<Br, C, F> Component for LoopUnit<C, F>
where
    C: Component<Output = Option<LoopControl<(), Br>>>,
    F: FnMut() -> C,
{
    type Output = Option<Br>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Some(control) = self.component.update(state, ctx, event) {
            match control {
                LoopControl::Continue(()) => {
                    self.component = (self.f)();
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

pub struct LoopMut<T, C, F> {
    value: T,
    component: C,
    f: F,
}
pub fn loop_mut<Co, Br, T, C, F>(init: Co, mut value: T, mut f: F) -> CF<LoopMut<T, C, F>>
where
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co, &mut T) -> C,
{
    cf(LoopMut {
        component: f(init, &mut value),
        value,
        f,
    })
}
impl<Co, Br, T, C, F> Component for LoopMut<T, C, F>
where
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co, &mut T) -> C,
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
                    self.component = (self.f)(co, &mut self.value);
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

/// Call a function on the current state returning a value which is yielded by
/// this component
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

/// Call a function on the current state returning a component.
/// This component then acts like the returned component.
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
            Self::F(_) => (),
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
            Self::F(_) => ctx.bounding_box.size(),
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

pub struct RenderState<S, F: Fn(&S, Ctx, &mut FrameBuffer)> {
    state: PhantomData<S>,
    f: F,
}
pub fn render_state<S, F: Fn(&S, Ctx, &mut FrameBuffer)>(f: F) -> CF<RenderState<S, F>> {
    cf(RenderState {
        state: PhantomData,
        f,
    })
}
impl<S, F> Component for RenderState<S, F>
where
    F: Fn(&S, Ctx, &mut FrameBuffer),
{
    type Output = ();
    type State = S;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        (self.f)(state, ctx, fb);
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

struct AndThenPersistentFirst<C, F> {
    component: C,
    f: F,
}
enum AndThenPersistentPriv<C, D, F> {
    // First is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First(Option<AndThenPersistentFirst<C, F>>),
    Second { component: D },
}
pub struct AndThenPersistent<C, D, F>(AndThenPersistentPriv<C, D, F>);

impl<T, U, C, D, F> Component for AndThenPersistent<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce(C, T) -> D,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        match &self.0 {
            AndThenPersistentPriv::First(first) => {
                first.as_ref().unwrap().component.render(state, ctx, fb)
            }
            AndThenPersistentPriv::Second { component, .. } => component.render(state, ctx, fb),
        }
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match &mut self.0 {
            AndThenPersistentPriv::First(first) => {
                match first.as_mut().unwrap().component.update(state, ctx, event) {
                    None => None,
                    Some(t) => {
                        let first = first.take().unwrap();
                        let mut second_component = (first.f)(first.component, t);
                        let peek_result = second_component.update(state, ctx, Event::Peek);
                        self.0 = AndThenPersistentPriv::Second {
                            component: second_component,
                        };
                        peek_result
                    }
                }
            }
            AndThenPersistentPriv::Second { component, .. } => component.update(state, ctx, event),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        match &self.0 {
            AndThenPersistentPriv::First(first) => {
                first.as_ref().unwrap().component.size(state, ctx)
            }
            AndThenPersistentPriv::Second { component, .. } => component.size(state, ctx),
        }
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
            Self::First { component, f } => match component.update(state, ctx, event) {
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

/// Similar to AndThen but the output of the component is ignored
pub enum Then<C, D, F> {
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First { component: C, f: Option<F> },
    Second(D),
}

impl<T, U, C, D, F> Component for Then<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce() -> D,
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
            Self::First { component, f } => match component.update(state, ctx, event) {
                None => None,
                Some(_) => {
                    let mut d = (f.take().unwrap())();
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

pub struct ThenSideEffect<C, F> {
    component: C,
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    f: Option<F>,
}

impl<T, C, F> Component for ThenSideEffect<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnOnce(&mut C::State),
{
    type Output = Option<T>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self.component.update(state, ctx, event) {
            None => None,
            Some(t) => {
                (self.f.take().expect("component yielded multiple times"))(state);
                Some(t)
            }
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct Map<C, F> {
    component: C,
    f: Option<F>,
}
impl<T, U, C, F> Component for Map<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnOnce(T) -> U,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self.component.update(state, ctx, event) {
            None => None,
            Some(t) => Some((self.f.take().expect("component yielded multiple times"))(
                t,
            )),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct MapVal<C, F> {
    component: C,
    f: Option<F>,
}
impl<T, U, C, F> Component for MapVal<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnOnce() -> U,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self.component.update(state, ctx, event) {
            None => None,
            Some(_) => Some((self.f.take().expect("component yielded multiple times"))()),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct Replace<C, T> {
    component: C,
    value: Option<T>,
}
impl<C, T, U> Component for Replace<C, T>
where
    C: Component<Output = Option<U>>,
{
    type Output = Option<T>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, ctx, event)
            .and_then(|_| self.value.take())
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

/// Component decorator that clears the frame buffer before rendering
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

/// Component decorator that yields `app::Exit` in response to a window close event,
/// and passes all other input to its child
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

/// Component decorator that yields `Err(Escape)` when the escape key is pressed
/// rather than passing the escape key event to its child
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
        match event {
            Event::Input(input::Input::Keyboard(_))
            | Event::Input(input::Input::Mouse(input::MouseInput::MousePress { .. })) => Some(()),
            #[cfg(feature = "gamepad")]
            Event::Input(input::Input::Gamepad(_)) => Some(()),
            _ => None,
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct OverlayTint<C: Component, D: Component, T: Tint> {
    pub foreground: C,
    pub background: D,
    pub tint: T,
    pub depth_delta: i8,
}

impl<C, D, T> Component for OverlayTint<C, D, T>
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
        self.background.update(state, ctx, event);
        self.foreground.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.foreground.size(state, ctx)
    }
}

pub struct Pause<C: Component>(C);

impl<T, C> Component for Pause<C>
where
    C: Component<Output = Option<T>>,
{
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        None
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct Some_<C: Component>(pub C);
impl<C: Component> Component for Some_<C> {
    type Output = Option<C::Output>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb)
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        Some(self.0.update(state, ctx, event))
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct None_<C: Component>(pub C);
impl<C: Component> Component for None_<C> {
    type Output = Option<C::Output>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb)
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.0.update(state, ctx, event);
        None
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct Unit<S> {
    state: PhantomData<S>,
}
pub fn unit<S>() -> CF<Unit<S>> {
    cf(Unit { state: PhantomData })
}
impl<S> Component for Unit<S> {
    type Output = ();
    type State = S;
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        ()
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(0, 0)
    }
}

pub struct Many<I, S> {
    iterable: I,
    state: PhantomData<S>,
}

impl<I, S, C> Component for Many<I, S>
where
    C: Component<State = S>,
    for<'a> &'a I: IntoIterator<Item = &'a C>,
    for<'a> &'a mut I: IntoIterator<Item = &'a mut C>,
{
    type Output = ();
    type State = S;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        for component in &self.iterable {
            component.render(state, ctx, fb);
        }
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        for component in &mut self.iterable {
            component.update(state, ctx, event);
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let mut size = Size::new_u16(0, 0);
        for component in &self.iterable {
            size = size.pairwise_max(component.size(state, ctx));
        }
        size
    }
}

pub fn many<I, S, C>(iterable: I) -> CF<Many<I, S>>
where
    C: Component<State = S>,
    for<'a> &'a I: IntoIterator<Item = &'a C>,
    for<'a> &'a mut I: IntoIterator<Item = &'a mut C>,
{
    cf(Many {
        iterable,
        state: PhantomData,
    })
}

pub fn styled_string<S>(string: String, style: Style) -> CF<IgnoreState<S, StyledString>> {
    cf(StyledString { string, style }).ignore_state()
}

pub struct OnInput<F, S> {
    f: F,
    state: PhantomData<S>,
}
impl<F, T, S> Component for OnInput<F, S>
where
    F: FnMut(input::Input) -> Option<T>,
{
    type Output = Option<T>;
    type State = S;
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Input(input) = event {
            if let Some(output) = (self.f)(input) {
                return Some(output);
            }
        }
        None
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}
pub fn on_input<F, T, S>(f: F) -> CF<OnInput<F, S>>
where
    F: FnMut(input::Input) -> Option<T>,
{
    cf(OnInput {
        f,
        state: PhantomData,
    })
}

pub struct OnInputState<F, S> {
    f: F,
    state: PhantomData<S>,
}
impl<F, T, S> Component for OnInputState<F, S>
where
    F: FnMut(input::Input, &mut S) -> Option<T>,
{
    type Output = Option<T>;
    type State = S;
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Input(input) = event {
            if let Some(output) = (self.f)(input, state) {
                return Some(output);
            }
        }
        None
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}
pub fn on_input_state<F, T, S>(f: F) -> CF<OnInputState<F, S>>
where
    F: FnMut(input::Input, &mut S) -> Option<T>,
{
    cf(OnInputState {
        f,
        state: PhantomData,
    })
}

pub struct WithTitle<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    pub title: T,
    pub component: C,
    pub padding: i32,
}

impl<T, C> WithTitle<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    fn component_ctx<'a>(&self, state: &T::State, ctx: Ctx<'a>) -> Ctx<'a> {
        ctx.add_y(self.padding + self.title.size(state, ctx).height() as i32)
    }
}

impl<T, C> Component for WithTitle<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    type Output = C::Output;
    type State = C::State;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.title.render(state, ctx, fb);
        self.component
            .render(state, self.component_ctx(state, ctx), fb);
    }

    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.title.update(state, ctx, event);
        self.component
            .update(state, self.component_ctx(state, ctx), event)
    }

    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let title_size = self.title.size(state, ctx);
        let component_size = self.component.size(state, self.component_ctx(state, ctx));
        let width = title_size.width().max(component_size.width());
        let height =
            (title_size.height() as i32 + component_size.height() as i32 + self.padding) as u32;
        Size::new(width, height)
    }
}

#[cfg(feature = "gamepad")]
mod gamepad {
    use chargrid_core::prelude::*;

    pub struct CatchEscapeOrStart<C: Component>(pub C);

    pub enum EscapeOrStart {
        Escape,
        Start,
    }

    pub type OrEscapeOrStart<T> = Result<T, EscapeOrStart>;

    impl<T, C> Component for CatchEscapeOrStart<C>
    where
        C: Component<Output = Option<T>>,
    {
        type Output = Option<OrEscapeOrStart<T>>;
        type State = C::State;
        fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
            self.0.render(state, ctx, fb);
        }
        fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
            match event {
                Event::Input(Input::Keyboard(input::keys::ESCAPE)) => {
                    Some(Err(EscapeOrStart::Escape))
                }
                Event::Input(Input::Gamepad(GamepadInput {
                    button: GamepadButton::Start,
                    ..
                })) => Some(Err(EscapeOrStart::Start)),
                _ => self.0.update(state, ctx, event).map(Ok),
            }
        }
        fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
            self.0.size(state, ctx)
        }
    }
}

#[cfg(feature = "gamepad")]
pub use gamepad::*;

pub mod boxed {
    pub use super::{boxed_cf, BoxedCF, Escape, LoopControl, OrEscape};
    use chargrid_core::{input, Component, Ctx, FrameBuffer, Style};

    pub fn val<S: 'static, T: 'static + Clone>(t: T) -> BoxedCF<Option<T>, S> {
        super::val(t).boxed_cf()
    }

    pub fn val_once<S: 'static, T: 'static>(t: T) -> BoxedCF<Option<T>, S> {
        super::val_once(t).boxed_cf()
    }

    pub fn break_<S: 'static, T: 'static, Co: 'static>(
        t: T,
    ) -> BoxedCF<Option<LoopControl<Co, T>>, S> {
        val_once(t).break_()
    }

    pub fn continue_<S: 'static, T: 'static, Br: 'static>(
        t: T,
    ) -> BoxedCF<Option<LoopControl<T, Br>>, S> {
        val_once(t).continue_()
    }

    pub fn never<S: 'static, T: 'static>() -> BoxedCF<Option<T>, S> {
        super::never().boxed_cf()
    }

    pub fn loop_state<S: 'static, Co, Br, C: 'static, F: 'static>(
        state: S,
        init: Co,
        f: F,
    ) -> BoxedCF<Option<Br>, ()>
    where
        C::State: Sized,
        C: Component<Output = Option<LoopControl<Co, Br>>, State = S>,
        F: FnMut(Co) -> C,
    {
        super::loop_state(state, init, f).boxed_cf()
    }

    pub fn loop_<Co, Br, C: 'static, F: 'static>(init: Co, f: F) -> BoxedCF<Option<Br>, C::State>
    where
        C::State: Sized,
        C: Component<Output = Option<LoopControl<Co, Br>>>,
        F: FnMut(Co) -> C,
    {
        super::loop_(init, f).boxed_cf()
    }

    pub fn loop_unit<Br, C: 'static, F: 'static>(f: F) -> BoxedCF<Option<Br>, C::State>
    where
        C::State: Sized,
        C: Component<Output = Option<LoopControl<(), Br>>>,
        F: FnMut() -> C,
    {
        super::loop_unit(f).boxed_cf()
    }

    pub fn loop_mut<Co, Br, T: 'static, C: 'static, F: 'static>(
        init: Co,
        value: T,
        f: F,
    ) -> BoxedCF<Option<Br>, C::State>
    where
        C::State: Sized,
        C: Component<Output = Option<LoopControl<Co, Br>>>,
        F: FnMut(Co, &mut T) -> C,
    {
        super::loop_mut(init, value, f).boxed_cf()
    }

    pub fn on_state<S: 'static, T: 'static, F: 'static>(f: F) -> BoxedCF<Option<T>, S>
    where
        F: FnOnce(&mut S) -> T,
    {
        super::on_state(f).boxed_cf()
    }

    pub fn on_state_then<C: 'static, F: 'static>(f: F) -> BoxedCF<C::Output, C::State>
    where
        C: Component,
        C::State: Sized,
        F: FnOnce(&mut C::State) -> C,
    {
        super::on_state_then(f).boxed_cf()
    }

    pub fn render<F: 'static + Fn(Ctx, &mut FrameBuffer)>(f: F) -> BoxedCF<(), ()> {
        super::render(f).boxed_cf()
    }

    pub fn render_state<S: 'static, F: 'static + Fn(&S, Ctx, &mut FrameBuffer)>(
        f: F,
    ) -> BoxedCF<(), S> {
        super::render_state(f).boxed_cf()
    }

    pub fn unit<S: 'static>() -> BoxedCF<(), S> {
        super::unit().boxed_cf()
    }

    pub fn many<I: 'static, S: 'static, C: 'static>(iterable: I) -> BoxedCF<(), S>
    where
        C: Component<State = S>,
        for<'a> &'a I: IntoIterator<Item = &'a C>,
        for<'a> &'a mut I: IntoIterator<Item = &'a mut C>,
    {
        super::many(iterable).boxed_cf()
    }

    pub fn styled_string<S: 'static>(string: String, style: Style) -> BoxedCF<(), S> {
        super::styled_string(string, style).boxed_cf()
    }

    pub fn on_input<F: 'static, T, S: 'static>(f: F) -> BoxedCF<Option<T>, S>
    where
        F: FnMut(input::Input) -> Option<T>,
    {
        super::on_input(f).boxed_cf()
    }

    pub fn on_input_state<F: 'static, T, S: 'static>(f: F) -> BoxedCF<Option<T>, S>
    where
        F: FnMut(input::Input, &mut S) -> Option<T>,
    {
        super::on_input_state(f).boxed_cf()
    }
}

#[macro_export]
macro_rules! either {
    ($type:ident = $first:ident) => {
        pub enum $type<$first> {
            $first($first),
        }
        impl<$first> Component for $type<$first>
            where
                $first: Component,
        {
            type Output = $first::Output;
            type State = $first::State;

            fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
                match self {
                    $type::$first(x) => x.render(state, ctx, fb),
                }
            }
            fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
                match self {
                    $type::$first(x) => x.update(state, ctx, event),
                }
            }
            fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
                match self {
                    $type::$first(x) => x.size(state, ctx),
                }
            }
        }
    };
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

#[macro_export]
macro_rules! boxed_many {
    ($($items:expr),* $(,)* ) => {
        $crate::control_flow::boxed::many([
            $($crate::control_flow::boxed_cf($items)),*
        ])
    };
}

pub use crate::{boxed_many, either, lens};
