pub use crate::control_flow::{
    ClickOut, Close, Escape, EscapeOrClickOut, EscapeOrStart, LoopControl, OrClickOut,
    OrEscapeOrClickOut,
};
use crate::{
    add_offset::AddOffset,
    align::{Align, Alignment},
    border::{Border, BorderStyle},
    bound_size::{BoundHeight, BoundSize, BoundWidth},
    control_flow::{boxed, Lens, LensState, OrClose, OrEscape, OrEscapeOrStart},
    fill::Fill,
    pad_by::{PadBy, Padding},
    pad_to::PadTo,
    set_size::{SetHeight, SetSize, SetWidth},
    text::StyledString,
};
use chargrid_core::{
    app, ctx_tint, input, Component, Coord, Ctx, Event, FrameBuffer, Rgba32, Size, Style, Tint,
    TintIdentity,
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

    pub fn with_title_vertical<T: Component<State = C::State>>(
        self,
        title: T,
        padding: i32,
    ) -> CF<WithTitleVertical<T, C>> {
        cf(WithTitleVertical {
            title,
            component: self.0,
            padding,
        })
    }

    pub fn with_title_horizontal<T: Component<State = C::State>>(
        self,
        title: T,
        padding: i32,
    ) -> CF<WithTitleHorizontal<T, C>> {
        cf(WithTitleHorizontal {
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

    pub fn and_then_side_effect<U, D, F>(self, f: F) -> CF<AndThenSideEffect<C, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce(T, &mut C::State) -> D,
    {
        cf(AndThenSideEffect::First {
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

    pub fn then_side_effect<U, D, F>(self, f: F) -> CF<ThenSideEffect<C, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce(&mut C::State) -> D,
    {
        cf(ThenSideEffect::First {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn side_effect<F>(self, f: F) -> CF<SideEffect<C, F>>
    where
        F: FnOnce(&mut C::State),
    {
        cf(SideEffect {
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

    pub fn map_side_effect<U, F>(self, f: F) -> CF<MapSideEffect<C, F>>
    where
        F: FnOnce(T, &mut C::State) -> U,
    {
        cf(MapSideEffect {
            component: self.0,
            f: Some(f),
        })
    }

    pub fn catch_escape(self) -> CF<CatchEscape<C>> {
        cf(CatchEscape(self.0))
    }

    pub fn catch_escape_or_start(self) -> CF<CatchEscapeOrStart<C>> {
        cf(CatchEscapeOrStart(self.0))
    }

    pub fn catch_click_out(self) -> CF<CatchClickOut<C>> {
        cf(CatchClickOut(self.0))
    }

    pub fn catch_escape_or_click_out(self) -> CF<CatchEscapeOrClickOut<C>> {
        cf(CatchEscapeOrClickOut(self.0))
    }

    pub fn menu_harness(self) -> CF<MenuHarness<C>> {
        cf(MenuHarness(self.0))
    }

    pub fn no_peek(self) -> CF<NoPeek<C>> {
        cf(NoPeek(self.0))
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

    pub fn on_each_tick<F: FnMut()>(self, f: F) -> CF<OnEachTick<C, F>> {
        cf(OnEachTick {
            component: self.0,
            f,
        })
    }

    pub fn on_each_tick_with_state<F: FnMut(&mut C::State)>(
        self,
        f: F,
    ) -> CF<OnEachTickWithState<C, F>> {
        cf(OnEachTickWithState {
            component: self.0,
            f,
        })
    }

    pub fn on_exit_with_state<F: FnMut(&mut C::State)>(self, f: F) -> CF<OnExitWithState<C, F>> {
        cf(OnExitWithState {
            component: self.0,
            f,
        })
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

impl<C: Component<Output = ()>> CF<C> {
    pub fn ignore_output<O>(self) -> CF<IgnoreOutput<O, C>> {
        cf(IgnoreOutput {
            output: PhantomData,
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
    pub fn boxed(self) -> boxed::CF<C::Output, C::State> {
        boxed::cf(self.0)
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
                    while let Some(control) =
                        self.component.update(&mut self.state, ctx, Event::Peek)
                    {
                        match control {
                            LoopControl::Continue(co) => {
                                self.component = (self.f)(co);
                            }
                            LoopControl::Break(br) => {
                                return Some(br);
                            }
                        }
                    }
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
                    while let Some(control) = self.component.update(state, ctx, Event::Peek) {
                        match control {
                            LoopControl::Continue(co) => {
                                self.component = (self.f)(co);
                            }
                            LoopControl::Break(br) => {
                                return Some(br);
                            }
                        }
                    }
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
                    while let Some(control) = self.component.update(state, ctx, Event::Peek) {
                        match control {
                            LoopControl::Continue(()) => {
                                self.component = (self.f)();
                            }
                            LoopControl::Break(br) => {
                                return Some(br);
                            }
                        }
                    }
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
                    while let Some(control) = self.component.update(state, ctx, Event::Peek) {
                        match control {
                            LoopControl::Continue(co) => {
                                self.component = (self.f)(co, &mut self.value);
                            }
                            LoopControl::Break(br) => {
                                return Some(br);
                            }
                        }
                    }

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

pub struct IgnoreOutput<O, C: Component<Output = ()>> {
    output: PhantomData<O>,
    component: C,
}

impl<O, C> Component for IgnoreOutput<O, C>
where
    C: Component<Output = ()>,
{
    type Output = Option<O>;
    type State = C::State;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(state, ctx, event);
        None
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
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

pub enum AndThenSideEffect<C, D, F> {
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First { component: C, f: Option<F> },
    Second(D),
}

impl<T, U, C, D, F> Component for AndThenSideEffect<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce(T, &mut C::State) -> D,
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
                    let mut d = (f.take().unwrap())(t, state);
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

pub enum ThenSideEffect<C, D, F> {
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First { component: C, f: Option<F> },
    Second(D),
}

impl<T, U, C, D, F> Component for ThenSideEffect<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce(&mut C::State) -> D,
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
                    let mut d = (f.take().unwrap())(state);
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

pub struct SideEffect<C, F> {
    component: C,
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    f: Option<F>,
}

impl<T, C, F> Component for SideEffect<C, F>
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

pub struct MapSideEffect<C, F> {
    component: C,
    f: Option<F>,
}
impl<T, U, C, F> Component for MapSideEffect<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnOnce(T, &mut C::State) -> U,
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
                t, state,
            )),
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
        let output_unless_close = self.0.update(state, ctx, event);
        if let Event::Input(input::Input::Keyboard(input::keys::ETX)) = event {
            return Some(app::Exit);
        }
        output_unless_close
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct OnExitWithState<C: Component, F: FnMut(&mut C::State)> {
    component: C,
    f: F,
}

impl<C: Component, F: FnMut(&mut C::State)> Component for OnExitWithState<C, F> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Input(input::Input::Keyboard(input::keys::ETX)) = event {
            (self.f)(state);
        }
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

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

pub struct CatchEscapeOrStart<C: Component>(pub C);

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
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => {
                Some(Err(EscapeOrStart::Escape))
            }
            #[cfg(feature = "gamepad")]
            Event::Input(input::Input::Gamepad(input::GamepadInput {
                button: input::GamepadButton::Start,
                ..
            })) => Some(Err(EscapeOrStart::Start)),
            _ => self.0.update(state, ctx, event).map(Ok),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct CatchClickOut<C: Component>(pub C);

impl<T, C> Component for CatchClickOut<C>
where
    C: Component<Output = Option<T>>,
{
    type Output = Option<OrClickOut<T>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let is_click_out = match event {
            Event::Input(input::Input::Mouse(input::MouseInput::MousePress { coord, .. })) => {
                let size = self.size(state, ctx);
                !ctx.bounding_box.set_size(size).contains_coord(coord)
            }
            _ => false,
        };
        if is_click_out {
            Some(Err(ClickOut))
        } else {
            self.0.update(state, ctx, event).map(Ok)
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct CatchEscapeOrClickOut<C: Component>(pub C);

impl<T, C> Component for CatchEscapeOrClickOut<C>
where
    C: Component<Output = Option<T>>,
{
    type Output = Option<OrEscapeOrClickOut<T>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let escape_or_click_out = match event {
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => {
                Some(EscapeOrClickOut::Escape)
            }
            Event::Input(input::Input::Mouse(input::MouseInput::MousePress { coord, .. })) => {
                let size = self.size(state, ctx);
                if ctx.bounding_box.set_size(size).contains_coord(coord) {
                    None
                } else {
                    Some(EscapeOrClickOut::ClickOut)
                }
            }
            _ => None,
        };
        if let Some(escape_or_click_out) = escape_or_click_out {
            Some(Err(escape_or_click_out))
        } else {
            self.0.update(state, ctx, event).map(Ok)
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct MenuHarness<C: Component>(pub C);
impl<T, C> Component for MenuHarness<C>
where
    C: Component<Output = Option<T>>,
{
    type Output = Option<OrClose<T>>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => Some(Err(Close)),
            #[cfg(feature = "gamepad")]
            Event::Input(input::Input::Gamepad(input::GamepadInput {
                button: input::GamepadButton::East,
                ..
            })) => Some(Err(Close)),
            #[cfg(feature = "gamepad")]
            Event::Input(input::Input::Gamepad(input::GamepadInput {
                button: input::GamepadButton::Start,
                ..
            })) => self
                .0
                .update(
                    state,
                    ctx,
                    Event::Input(input::Input::Keyboard(input::keys::RETURN)),
                )
                .map(Ok),
            _ => self.0.update(state, ctx, event).map(Ok),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

pub struct NoPeek<C>(C);
impl<T, C: Component<Output = Option<T>>> Component for NoPeek<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Event::Peek = event {
            return None;
        }
        self.0.update(state, ctx, event)
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

pub struct WithTitleVertical<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    pub title: T,
    pub component: C,
    pub padding: i32,
}

impl<T, C> WithTitleVertical<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    fn component_ctx<'a>(&self, state: &T::State, ctx: Ctx<'a>) -> Ctx<'a> {
        ctx.add_y(self.padding + self.title.size(state, ctx).height() as i32)
    }
}

impl<T, C> Component for WithTitleVertical<T, C>
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

pub struct WithTitleHorizontal<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    pub title: T,
    pub component: C,
    pub padding: i32,
}

impl<T, C> WithTitleHorizontal<T, C>
where
    T: Component,
    C: Component<State = T::State>,
{
    fn component_ctx<'a>(&self, state: &T::State, ctx: Ctx<'a>) -> Ctx<'a> {
        ctx.add_x(self.padding + self.title.size(state, ctx).width() as i32)
    }
}

impl<T, C> Component for WithTitleHorizontal<T, C>
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
        let width =
            (title_size.width() as i32 + component_size.width() as i32 + self.padding) as u32;
        let height = title_size.height().max(component_size.height());
        Size::new(width, height)
    }
}

pub struct OnEachTick<C: Component, F: FnMut()> {
    component: C,
    f: F,
}
impl<C: Component, F: FnMut()> Component for OnEachTick<C, F> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        (self.f)();
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

pub struct OnEachTickWithState<C: Component, F: FnMut(&mut C::State)> {
    component: C,
    f: F,
}
impl<C: Component, F: FnMut(&mut C::State)> Component for OnEachTickWithState<C, F> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        (self.f)(state);
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}
