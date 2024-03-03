pub use crate::control_flow::LoopControl;
use crate::{
    align::Alignment,
    border::BorderStyle,
    control_flow::{
        unboxed, Lens, OrClickOut, OrClose, OrEscape, OrEscapeOrClickOut, OrEscapeOrStart,
    },
    pad_by::Padding,
};
pub use chargrid_core::app;
use chargrid_core::{
    input, BoxedComponent, Component, Coord, Ctx, Event, FrameBuffer, Rgba32, Size, Style, Tint,
};
use std::time::Duration;

/// A component which exposes additional methods for sequencing control flow operations. For
/// components which notionally yield a single value of type `T` (e.g. a menu from which a single
/// selection may be made), the convention is to have their `CF` yield a result of `Option<T>`.
/// In such a case, a value of `None` will be yielded in response to all input until the component
/// "completes" (e.g. in the menu example, until the selection is made), at which point `Some(...)`
/// will be yielded containing a value representing the result of the component. In these cases,
/// unless specified otherwise, it's considered unspecified behaviour to present a completed
/// component with input events.
pub struct CF<O, S>(unboxed::CF<BoxedComponent<O, S>>);

/// Build a new `CF` from an existing component
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
    /// Change the expected `State` of `self` by applying a `Lens`.
    pub fn lens_state<S_: 'static, L: 'static>(self, lens: L) -> CF<O, S_>
    where
        L: Lens<Input = S_, Output = S>,
    {
        self.0.lens_state(lens).boxed()
    }

    /// Convenience method to change the result type to an option whose
    /// value is `Some(<existing value>)`
    pub fn some(self) -> CF<Option<O>, S> {
        self.0.some().boxed()
    }

    /// Convenience method to change the result type to an option whose
    /// value is `None`
    pub fn none(self) -> CF<Option<O>, S> {
        self.0.none().boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self`, but which
    /// is rendered over a passive component `background` with a given
    /// `Tint` applied when rendered (`self` is rendered as normal)
    pub fn overlay_tint<D: 'static + Component<State = S>, T: 'static + Tint>(
        self,
        background: D,
        tint: T,
        depth_delta: i8,
    ) -> Self {
        self.0.overlay_tint(background, tint, depth_delta).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self`, but which
    /// is rendered over a passive component `background`
    pub fn overlay<D: 'static + Component<State = S>>(
        self,
        background: D,
        depth_delta: i8,
    ) -> Self {
        self.0.overlay(background, depth_delta).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but the
    /// frame is cleared before rendering. Useful as the outer-most layer
    /// of an application
    pub fn clear_each_frame(self) -> Self {
        self.0.clear_each_frame().boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but whose
    /// background has been filled in with a given colour
    pub fn fill(self, background: Rgba32) -> Self {
        self.0.fill(background).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but with
    /// a border of specified style
    pub fn border(self, style: BorderStyle) -> Self {
        self.0.border(style).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// padded to a given size
    pub fn pad_to(self, size: Size) -> Self {
        self.0.pad_to(size).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// padded by a given size
    pub fn pad_by(self, padding: Padding) -> Self {
        self.0.pad_by(padding).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with a specified alignment
    pub fn align(self, alignment: Alignment) -> Self {
        self.0.align(alignment).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// centered within its parent component
    pub fn centre(self) -> Self {
        self.0.centre().boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with a specified offset applied when rendsering
    pub fn add_offset(self, offset: Coord) -> Self {
        self.0.add_offset(offset).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with a specified horizontal offset
    pub fn add_x(self, x: i32) -> Self {
        self.0.add_x(x).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with a specified vertical offset
    pub fn add_y(self, y: i32) -> Self {
        self.0.add_y(y).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose size has been overridden to a specific value
    pub fn set_size(self, size: Size) -> Self {
        self.0.set_size(size).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose width has been overridden to a specific value
    pub fn set_width(self, width: u32) -> Self {
        self.0.set_width(width).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose height has been overridden to a specific value
    pub fn set_height(self, height: u32) -> Self {
        self.0.set_height(height).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose size is bounded by some minimum size
    pub fn bound_size(self, size: Size) -> Self {
        self.0.bound_size(size).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose width is bounded by some minimum width
    pub fn bound_width(self, width: u32) -> Self {
        self.0.bound_width(width).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// whose height is bounded by some minimum height
    pub fn bound_height(self, height: u32) -> Self {
        self.0.bound_height(height).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with an additional component rendered on top with a specified
    /// padding. Useful for adding title text to components
    pub fn with_title_vertical<T: 'static + Component<State = S>>(
        self,
        title: T,
        padding: i32,
    ) -> Self {
        self.0.with_title_vertical(title, padding).boxed()
    }

    /// Returns a new `CF` with identical behaviour to `self` but
    /// with an additional component rendered to the left with a specified
    /// padding. Useful for adding labels to fields.
    pub fn with_title_horizontal<T: 'static + Component<State = S>>(
        self,
        title: T,
        padding: i32,
    ) -> Self {
        self.0.with_title_horizontal(title, padding).boxed()
    }
}

impl<O: 'static, S: 'static> CF<O, S> {
    /// Internalizes the expected `State` of a `CF` to a given value, and
    /// produces a new `CF` expecting no external state (ie. it expects `()`
    /// as its external state). The specified value will be presented as the
    /// `State` to the callee.
    pub fn with_state(self, state: S) -> CF<O, ()> {
        self.0.with_state(state).boxed()
    }
}

impl<S: 'static> CF<(), S> {
    /// Takes a `CF` which yields no value, and creates a new `CF` which renders the same,
    /// but which yields `None` until a specified `Duration` has passed, at which point it
    /// yields `Some(())`
    pub fn delay(self, duration: Duration) -> CF<Option<()>, S> {
        self.0.delay(duration).boxed()
    }

    /// Takes a `CF` which yields no value, and creates a new `CF` which renders the same,
    /// but which yields `None` until a key is pressed, at which point it yields `Some(())`
    pub fn press_any_key(self) -> CF<Option<()>, S> {
        self.0.press_any_key().boxed()
    }
}

impl<T: 'static, S: 'static> CF<Option<T>, S> {
    /// Creates a `CF` which behaves the same as `self` until `self` completes, at which point a
    /// given function is invoked on `self` and the result of `self` completing. An example of when
    /// this is necessary is a menu from which a single selection can be made, but which preserves
    /// the most-recently highlighted entry the next time it is opened. One could store the state
    /// representing the currently-highlighted entry in the menu component itself, and have the
    /// menu yield the selection when the component completes. Since `f` is invoked on the
    /// component as well as its result, it's free to leak the component through its return value
    /// so the menu can be re-invoked with its internal state preserved. This is only valid for
    /// components which may complete multiple times, of which `chargrid_common::menu::MenuCF` is
    /// an example.
    pub fn and_then_persistent<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(Self, T) -> D,
    {
        self.0.and_then_persistent(|a, b| f(a.into(), b)).boxed()
    }

    /// Invoke a function `f` on the result of `self` when it completes. The given function returns
    /// a new component which replaces `self`. Use to sequence multiple components one after the
    /// other.
    pub fn and_then<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(T) -> D,
    {
        self.0.and_then(f).boxed()
    }

    /// Invoke a function `f` on the result of `self` when it completes. The given function returns
    /// a new component which replaces `self`. Use to sequence multiple components one after the
    /// other. Unlike with `and_then`, the given function is also passed a mutable reference to the
    /// state.
    pub fn and_then_side_effect<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(T, &mut S) -> D,
    {
        self.0.and_then_side_effect(f).boxed()
    }

    /// Invoke a function `f` after `self` completes, ignoring the result. The given function
    /// returns a new component which replaces `self`.
    pub fn then<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce() -> D,
    {
        self.0.then(f).boxed()
    }

    /// Invoke a function `f` after `self` completes, ignoring the result. The given function
    /// returns a new component which replaces `self`. Unlike with `then`, the given function is
    /// also passed a mutable reference to the state.
    pub fn then_side_effect<U, D: 'static, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        D: Component<Output = Option<U>, State = S>,
        F: FnOnce(&mut S) -> D,
    {
        self.0.then_side_effect(f).boxed()
    }

    /// Invoke a function `f` on a mutable reference to the external state, after `self` completes.
    /// The behaviour of `self` is otherwise unchanged.
    pub fn side_effect<F: 'static>(self, f: F) -> CF<Option<T>, S>
    where
        F: FnOnce(&mut S),
    {
        self.0.side_effect(f).boxed()
    }

    /// Creates a new `CF` which invokes a given function `f` on the result of `self`, yielding the
    /// result
    pub fn map<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce(T) -> U,
    {
        self.0.map(f).boxed()
    }

    /// Creates a new `CF` which invokes a given function `f` after `self` completes (ignoring ites
    /// result), yielding the result of `f`
    pub fn map_val<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce() -> U,
    {
        self.0.map_val(f).boxed()
    }

    /// Creates a new `CF` which invokes a given function `f` on the result of `self`, yielding the
    /// result. Unlike with `map`, the given function is also passed a mutable reference to the
    /// external state.
    pub fn map_side_effect<U, F: 'static>(self, f: F) -> CF<Option<U>, S>
    where
        F: FnOnce(T, &mut S) -> U,
    {
        self.0.map_side_effect(f).boxed()
    }

    /// Creates a new `CF` which may be interrupted by the user pressing the escape key
    pub fn catch_escape(self) -> CF<Option<OrEscape<T>>, S> {
        self.0.catch_escape().boxed()
    }

    /// Creates a new `CF` which may be interrupted by the user pressing the escape key or the
    /// start button on a gamepad
    pub fn catch_escape_or_start(self) -> CF<Option<OrEscapeOrStart<T>>, S> {
        self.0.catch_escape_or_start().boxed()
    }

    /// Creates a new `CF` which may be interrupted by the user clicking outside the component
    pub fn catch_click_out(self) -> CF<Option<OrClickOut<T>>, S> {
        self.0.catch_click_out().boxed()
    }

    /// Creates a new `CF` which may be interrupted by the user clicking outside the component or
    /// pressing escape
    pub fn catch_escape_or_click_out(self) -> CF<Option<OrEscapeOrClickOut<T>>, S> {
        self.0.catch_escape_or_click_out().boxed()
    }

    /// Applies a common policy for interracting with menus. Intercepts the escape and "east"
    /// gamepad button (the right-most symbol button on most gamepads) and yields
    /// `Some(Err(Close))` in response, and intercepts the "start" gamepad button and injects a
    /// "return" keypress in its place.
    pub fn menu_harness(self) -> CF<Option<OrClose<T>>, S> {
        self.0.menu_harness().boxed()
    }

    /// Prevents `Event::Peek` events from propagating to `self`
    pub fn no_peek(self) -> CF<Option<T>, S> {
        self.0.no_peek().boxed()
    }

    /// Creates a new `CF` with the same behaviour as `self` but whose yielded result is replaced
    /// with a given result
    pub fn replace<U: 'static>(self, value: U) -> CF<Option<U>, S> {
        self.0.replace(value).boxed()
    }

    /// Convenience combinator for use in a loop context (e.g. in the argument to `repeat`).
    /// Creates a new `CF` which behaves the same as `self` but whose result is wrapped in
    /// `LoopControl::Continue`
    pub fn continue_<Br: 'static>(self) -> CF<Option<LoopControl<T, Br>>, S> {
        self.0.continue_().boxed()
    }

    /// Convenience combinator for use in a loop context (e.g. in the argument to `repeat`).
    /// Creates a new `CF` which behaves the same as `self` but whose result is wrapped in
    /// `LoopControl::Break`
    pub fn break_<Co: 'static>(self) -> CF<Option<LoopControl<Co, T>>, S> {
        self.0.break_().boxed()
    }

    /// Convenience combinator for use in a loop context (e.g. in the argument to `repeat`).
    /// Creates a new `CF` which behaves the same as `self` but whose result is replaced by
    /// `LoopControl::Continue(value)`
    pub fn continue_with<Br: 'static, U: 'static>(
        self,
        value: U,
    ) -> CF<Option<LoopControl<U, Br>>, S> {
        self.0.continue_with(value).boxed()
    }

    /// Convenience combinator for use in a loop context (e.g. in the argument to `repeat`).
    /// Creates a new `CF` which behaves the same as `self` but whose result is replaced by
    /// `LoopControl::Break(value)`
    pub fn break_with<Co: 'static, U: 'static>(
        self,
        value: U,
    ) -> CF<Option<LoopControl<Co, U>>, S> {
        self.0.break_with(value).boxed()
    }

    /// For components which may complete multiple times (e.g. menus) this combinator repeatedly
    /// invokes the component, and calls the provided function `f` on some accumulator value, and
    /// the result of the component. The provided function returns a `LoopControl` which determines
    /// whether to continue repeating.
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

    /// For components which may complete multiple times (e.g. menus) this combinator repeatedly
    /// invokes the component, and calls the provided function `f` on the result of the component.
    /// The provided function returns a `LoopControl` which determines whether to continue
    /// repeating.
    pub fn repeat_unit<O: 'static, F: 'static>(self, mut f: F) -> CF<Option<O>, S>
    where
        F: FnMut(T) -> CF<Option<LoopControl<(), O>>, S>,
    {
        self.repeat((), move |(), entry| f(entry))
    }

    /// Prevent a component from receiving any events.
    pub fn pause(self) -> CF<Option<T>, S> {
        self.0.pause().boxed()
    }

    /// Call a given function on each tick of the component.
    pub fn on_each_tick<F: FnMut() + 'static>(self, f: F) -> Self {
        self.0.on_each_tick(f).boxed()
    }

    /// Call a given function on the state on each tick of the component.
    pub fn on_each_tick_with_state<F: FnMut(&mut S) + 'static>(self, f: F) -> Self {
        self.0.on_each_tick_with_state(f).boxed()
    }

    /// Call a function on the state when the window is closed.
    pub fn on_exit_with_state<F: FnMut(&mut S) + 'static>(self, f: F) -> Self {
        self.0.on_exit_with_state(f).boxed()
    }
}

impl<O: 'static> CF<O, ()> {
    /// For components that don't depend on external state, this combinator creates a new `CF` that
    /// accepts any state.
    pub fn ignore_state<S: 'static>(self) -> CF<O, S> {
        self.0.ignore_state().boxed()
    }
}

impl<S: 'static> CF<(), S> {
    /// For components that yield `()`, creates a new `CF` that claims to yield `Option<T>` for any
    /// `T`, but which always yields `None`.
    pub fn ignore_output<O: 'static>(self) -> CF<Option<O>, S> {
        self.0.ignore_output().boxed()
    }
}

impl<S: 'static> CF<app::Output, S> {
    /// Yields `Some(app::Exit)` when a window closed event is received. This typically annotates
    /// the outside of an application so that application doesn't need to deal with window close
    /// events.
    pub fn exit_on_close(self) -> Self {
        self.0.exit_on_close().boxed()
    }
}

/// Convenience type alias for most applications. There is no external state, and the yielded
/// result is an app control flow signal (typically whether the application wishes to exit).
pub type App = CF<app::Output, ()>;

/// Creates a `CF` which always yields clones of a given value
pub fn val<S: 'static, T: 'static + Clone>(t: T) -> CF<Option<T>, S> {
    unboxed::val(t).boxed()
}

/// Creates a `CF` which yields a given value once. Should not be ticked multiple times
pub fn val_once<S: 'static, T: 'static>(t: T) -> CF<Option<T>, S> {
    unboxed::val_once(t).boxed()
}

/// Creates a `CF` which yields a given value once, wrapped in `LoopControl::Break`. Should not be
/// ticked multiple times
pub fn break_<S: 'static, T: 'static, Co: 'static>(t: T) -> CF<Option<LoopControl<Co, T>>, S> {
    val_once(t).break_()
}

/// Creates a `CF` which yields a given value once, wrapped in `LoopControl::Continue`. Should not
/// be ticked multiple times
pub fn continue_<S: 'static, T: 'static, Br: 'static>(t: T) -> CF<Option<LoopControl<T, Br>>, S> {
    val_once(t).continue_()
}

/// Creates a new `CF` which never completes
pub fn never<S: 'static, T: 'static>() -> CF<Option<T>, S> {
    unboxed::never().boxed()
}

/// Repeatedly invoke the component-producing function `f` on the result of the component produced
/// by `f`'s its previous invocation. The external state of the components produced by `f` is
/// passed in as the `state` argument, and the component returned by `loop_state` expects no
/// external state.
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

/// Repeatedly invoke the component-producing function `f` on the result of the component produced
/// by `f`'s its previous invocation.
pub fn loop_<Co, Br, C: 'static, F: 'static>(init: Co, f: F) -> CF<Option<Br>, C::State>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<Co, Br>>>,
    F: FnMut(Co) -> C,
{
    unboxed::loop_(init, f).boxed()
}

/// Repeatedly invoke the component-producing function `f` until its produced component yields
/// `LoopControl::Break`.
pub fn loop_unit<Br, C: 'static, F: 'static>(f: F) -> CF<Option<Br>, C::State>
where
    C::State: Sized,
    C: Component<Output = Option<LoopControl<(), Br>>>,
    F: FnMut() -> C,
{
    unboxed::loop_unit(f).boxed()
}

/// Repeatedly invoke the component-producing function `f` until its produced component yields
/// `LoopControl::Break`. The provided function is also passed a mutable reference to a state.
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
