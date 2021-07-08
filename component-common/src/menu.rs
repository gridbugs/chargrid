use chargrid_component::*;
use std::collections::HashMap;

pub struct MenuItemChoice<T: Clone, S> {
    pub identifier_selected: Box<dyn Component<State = S, Output = ()>>,
    pub identifier_deselected: Box<dyn Component<State = S, Output = ()>>,
    pub value: T,
}

pub enum MenuItem<T: Clone, S> {
    Choice(MenuItemChoice<T, S>),
    Space,
}

pub struct Menu<T: Clone, S> {
    items: Vec<MenuItem<T, S>>,
    selected_index: usize,
    hotkeys: Option<HashMap<input::KeyboardInput, T>>,
    #[cfg(feature = "gamepad")]
    gamepad_hotkeys: Option<HashMap<input::GamepadButton, T>>,
}

impl<T: Clone, S> Component for Menu<T, S> {
    type State = S;
    type Output = Option<T>;
    fn render(&self, state: &S, ctx: Ctx, fb: &mut FrameBuffer) {}
    fn update(&mut self, state: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        None
    }
}
