pub use crate::control_flow::{lens, Close, Escape, EscapeOrStart, LensFns, LoopControl};
use crate::{
    align::Alignment,
    border::BorderStyle,
    control_flow::{unboxed, Lens, OrClose, OrEscape, OrEscapeOrStart},
    pad_by::Padding,
};
pub use chargrid_core::app;
use chargrid_core::{
    input, BoxedComponent, Component, Coord, Ctx, Event, FrameBuffer, Rgba32, Size, Style, Tint,
};
use std::time::Duration;

pub struct CF<O, S>(unboxed::CF<BoxedComponent<O, S>>);

pub fn cf<C: 'static + Component>(component: C) -> CF<C::Output, C::State>
where
    C::State: Sized,
{
    CF(unboxed::cf(BoxedComponent(Box::new(component))))
}

impl<O, S> From<BoxedComponent<O, S>> for CF<O, S> {
    fn from(boxed_component: BoxedComponent<O, S>) -> Self {
        CF(unboxed::cf(boxed_component))
    }
}

impl<O, S> Component for CF<O, S> {
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

impl<O: 'static, S: 'static> CF<O, S> {
    pub fn lens_state<S_: 'static, L: 'static>(self, lens: L) -> CF<O, S_>
    where
        L: Lens<Input = S_, Output = S>,
    {
        self.0.lens_state(lens).boxed()
    }

    pub fn some(self) -> CF<Option<O>, S> {
        self.0.some().boxed()
    }

    pub fn none(self) -> CF<Option<O>, S> {
        self.0.none().boxed()
    }

    pub fn overlay_tint<D: 'static + Component<State = S>, T: 'static + Tint>(
        self,
        background: D,
        tint: T,
        depth_delta: i8,
    ) -> Self {
        self.0.overlay_tint(background, tint, depth_delta).boxed()
    }

    pub fn overlay<D: 'static + Component<State = S>>(
        self,
        background: D,
        depth_delta: i8,
    ) -> Self {
        self.0.overlay(background, depth_delta).boxed()
    }

    pub fn clear_each_frame(self) -> Self {
        self.0.clear_each_frame().boxed()
    }

    pub fn fill(self, background: Rgba32) -> Self {
        self.0.fill(background).boxed()
    }

    pub fn border(self, style: BorderStyle) -> Self {
        self.0.border(style).boxed()
    }

    pub fn pad_to(self, size: Size) -> Self {
        self.0.pad_to(size).boxed()
    }

    pub fn pad_by(self, padding: Padding) -> Self {
        self.0.pad_by(padding).boxed()
    }

    pub fn align(self, alignment: Alignment) -> Self {
        self.0.align(alignment).boxed()
    }

    pub fn centre(self) -> Self {
        self.0.centre().boxed()
    }

    pub fn add_offset(self, offset: Coord) -> Self {
        self.0.add_offset(offset).boxed()
    }

    pub fn add_x(self, x: i32) -> Self {
        self.0.add_x(x).boxed()
    }

    pub fn add_y(self, y: i32) -> Self {
        self.0.add_y(y).boxed()
    }

    pub fn set_size(self, size: Size) -> Self {
        self.0.set_size(size).boxed()
    }

    pub fn set_width(self, width: u32) -> Self {
        self.0.set_width(width).boxed()
    }

    pub fn set_height(self, height: u32) -> Self {
        self.0.set_height(height).boxed()
    }

    pub fn bound_size(self, size: Size) -> Self {
        self.0.bound_size(size).boxed()
    }

    pub fn bound_width(self, width: u32) -> Self {
        self.0.bound_width(width).boxed()
    }

    pub fn bound_height(self, height: u32) -> Self {
        self.0.bound_height(height).boxed()
    }

    pub fn with_title<T: 'static + Component<State = S>>(self, title: T, padding: i32) -> Self {
        self.0.with_title(title, padding).boxed()
    }
}

impl<O: 'static, S: 'static> CF<O, S> {
    pub fn with_state(self, state: S) -> CF<O, ()> {
        self.0.with_state(state).boxed()
    }
}

impl<S: 'static> CF<(), S> {
    pub fn delay(self, duration: Duration) -> CF<Option<()>, S> {
        self.0.delay(duration).boxed()
    }

    pub fn press_any_key(self) -> CF<Option<()>, S> {
        self.0.press_any_key().boxed()
    }
}

impl<T: 'static, S: 'static> CF<Option<T>, S> {
    pub fn and_then_persistent<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(Self, T) -> D,
    {
        self.0.and_then_persistent(|a, b| f(a.into(), b)).boxed()
    }

    pub fn and_then<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(T) -> D,
    {
        self.0.and_then(f).boxed()
    }

    pub fn and_then_side_effect<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(T, &mut S) -> D,
    {
        self.0.and_then_side_effect(f).boxed()
    }

    pub fn then<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce() -> D,
    {
        self.0.then(f).boxed()
    }

    pub fn then_side_effect<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(&mut S) -> D,
    {
        self.0.then_side_effect(f).boxed()
    }

    pub fn side_effect<F: 'static>(self, f: F) -> CF<Option<T>, S>
    where
        F: FnOnce(&mut S),
    {
        self.0.side_effect(f).boxed()
    }

    pub fn map<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce(T) -> U,
    {
        self.0.map(f).boxed()
    }

    pub fn map_val<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce() -> U,
    {
        self.0.map_val(f).boxed()
    }

    pub fn map_side_effect<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce(T, &mut S) -> U,
    {
        self.0.map_side_effect(f).boxed()
    }

    pub fn catch_escape(self) -> CF<Option<OrEscape<T>>, S> {
        self.0.catch_escape().boxed()
    }

    pub fn catch_escape_or_start(self) -> CF<Option<OrEscapeOrStart<T>>, S> {
        self.0.catch_escape_or_start().boxed()
    }

    pub fn menu_harness(self) -> CF<Option<OrClose<T>>, S> {
        self.0.menu_harness().boxed()
    }

    pub fn no_peek(self) -> CF<Option<T>, S> {
        self.0.no_peek().boxed()
    }

    pub fn replace<U: 'static>(self, value: U) -> CF<Option<U>, S> {
        self.0.replace(value).boxed()
    }

    pub fn continue_<Br: 'static>(self) -> CF<Option<LoopControl<T, Br>>, S> {
        self.0.continue_().boxed()
    }

    pub fn break_<Co: 'static>(self) -> CF<Option<LoopControl<Co, T>>, S> {
        self.0.break_().boxed()
    }

    pub fn continue_with<Br: 'static, U: 'static>(
        self,
        value: U,
    ) -> CF<Option<LoopControl<U, Br>>, S> {
        self.0.continue_with(value).boxed()
    }

    pub fn break_with<Co: 'static, U: 'static>(
        self,
        value: U,
    ) -> CF<Option<LoopControl<Co, U>>, S> {
        self.0.break_with(value).boxed()
    }

    pub fn repeat<A: 'static, O: 'static, F: 'static>(self, init: A, f: F) -> CF<Option<O>, S>
    where
        F: FnMut(A, T) -> CF<Option<LoopControl<A, O>>, S>,
    {
        loop_((self, f, init), |(self_, mut f, acc)| {
            self_.and_then_persistent(|self_, entry| {
                f(acc, entry).map(|loop_control| loop_control.map_continue(|c| (self_, f, c)))
            })
        })
    }

    pub fn repeat_unit<O: 'static, F: 'static>(self, mut f: F) -> CF<Option<O>, S>
    where
        F: FnMut(T) -> CF<Option<LoopControl<(), O>>, S>,
    {
        self.repeat((), move |(), entry| f(entry))
    }

    pub fn pause(self) -> CF<Option<T>, S> {
        self.0.pause().boxed()
    }
}

impl<O: 'static> CF<O, ()> {
    pub fn ignore_state<S: 'static>(self) -> CF<O, S> {
        self.0.ignore_state().boxed()
    }
}

impl<S: 'static> CF<(), S> {
    pub fn ignore_output<O: 'static>(self) -> CF<Option<O>, S> {
        self.0.ignore_output().boxed()
    }
}

impl<S: 'static> CF<app::Output, S> {
    pub fn exit_on_close(self) -> Self {
        self.0.exit_on_close().boxed()
    }
}

pub type App = CF<app::Output, ()>;

pub fn val<S: 'static, T: 'static + Clone>(t: T) -> CF<Option<T>, S> {
    unboxed::val(t).boxed()
}

pub fn val_once<S: 'static, T: 'static>(t: T) -> CF<Option<T>, S> {
    unboxed::val_once(t).boxed()
}

pub fn break_<S: 'static, T: 'static, Co: 'static>(t: T) -> CF<Option<LoopControl<Co, T>>, S> {
    val_once(t).break_()
}

pub fn continue_<S: 'static, T: 'static, Br: 'static>(t: T) -> CF<Option<LoopControl<T, Br>>, S> {
    val_once(t).continue_()
}

pub fn never<S: 'static, T: 'static>() -> CF<Option<T>, S> {
    unboxed::never().boxed()
}

pub fn loop_state<S: 'static, Co, Br, C: 'static, F: 'static>(
    state: S,
    init: Co,
    f: F,
) -> CF<Option<Br>, ()>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<Co, Br>>, State = S>,
    F: FnMut(Co) -> C,
{
    unboxed::loop_state(state, init, f).boxed()
}

pub fn loop_<Co, Br, C: 'static, F: 'static>(init: Co, f: F) -> CF<Option<Br>, C::State>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co) -> C,
{
    unboxed::loop_(init, f).boxed()
}

pub fn loop_unit<Br, C: 'static, F: 'static>(f: F) -> CF<Option<Br>, C::State>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<(), Br>>>,
    F: FnMut() -> C,
{
    unboxed::loop_unit(f).boxed()
}

pub fn loop_mut<Co, Br, T: 'static, C: 'static, F: 'static>(
    init: Co,
    value: T,
    f: F,
) -> CF<Option<Br>, C::State>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co, &mut T) -> C,
{
    unboxed::loop_mut(init, value, f).boxed()
}

pub fn on_state<S: 'static, T: 'static, F: 'static>(f: F) -> CF<Option<T>, S>
where
    F: FnOnce(&mut S) -> T,
{
    unboxed::on_state(f).boxed()
}

pub fn on_state_then<C: 'static, F: 'static>(f: F) -> CF<C::Output, C::State>
where
    C: Component,
    C::State: Sized,
    F: FnOnce(&mut C::State) -> C,
{
    unboxed::on_state_then(f).boxed()
}

pub fn render<F: 'static + Fn(Ctx, &mut FrameBuffer)>(f: F) -> CF<(), ()> {
    unboxed::render(f).boxed()
}

pub fn render_state<S: 'static, F: 'static + Fn(&S, Ctx, &mut FrameBuffer)>(f: F) -> CF<(), S> {
    unboxed::render_state(f).boxed()
}

pub fn unit<S: 'static>() -> CF<(), S> {
    unboxed::unit().boxed()
}

pub fn many<I: 'static, S: 'static, C: 'static>(iterable: I) -> CF<(), S>
where
    C: Component<State = S>,
    for<'a> &'a I: IntoIterator<Item = &'a C>,
    for<'a> &'a mut I: IntoIterator<Item = &'a mut C>,
{
    unboxed::many(iterable).boxed()
}

pub fn styled_string<S: 'static>(string: String, style: Style) -> CF<(), S> {
    unboxed::styled_string(string, style).boxed()
}

pub fn on_input<F: 'static, T, S: 'static>(f: F) -> CF<Option<T>, S>
where
    F: FnMut(input::Input) -> Option<T>,
{
    unboxed::on_input(f).boxed()
}

pub fn on_input_state<F: 'static, T, S: 'static>(f: F) -> CF<Option<T>, S>
where
    F: FnMut(input::Input, &mut S) -> Option<T>,
{
    unboxed::on_input_state(f).boxed()
}
