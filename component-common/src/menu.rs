use chargrid_component::*;
use std::collections::HashMap;

pub struct MenuItemChoice<T: Clone, S = ()> {
    pub identifier_selected: Box<dyn Component<State = S, Output = ()>>,
    pub identifier_deselected: Box<dyn Component<State = S, Output = ()>>,
    pub value: T,
}

pub enum MenuItem<T: Clone, S = ()> {
    Choice(MenuItemChoice<T, S>),
    Space,
}

pub struct Menu<T: Clone, S = ()> {
    items: Vec<MenuItem<T, S>>,
    selected_index: usize,
    hotkeys: Option<HashMap<input::KeyboardInput, T>>,
    #[cfg(feature = "gamepad")]
    gamepad_hotkeys: Option<HashMap<input::GamepadButton, T>>,
}

pub struct MenuBuilder<T: Clone, S = ()> {
    pub items: Vec<MenuItem<T, S>>,
    pub selected_index: usize,
    pub hotkeys: Option<HashMap<input::KeyboardInput, T>>,
    #[cfg(feature = "gamepad")]
    pub gamepad_hotkeys: Option<HashMap<input::GamepadButton, T>>,
}

impl<T: Clone, S> Default for MenuBuilder<T, S> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            hotkeys: None,
            #[cfg(feature = "gamepad")]
            gamepad_hotkeys: None,
        }
    }
}

impl<T: Clone, S> MenuBuilder<T, S> {
    pub fn add_choice(self) -> Self {
        self
    }
    pub fn build(self) -> Menu<T, S> {
        let Self {
            items,
            selected_index,
            hotkeys,
            #[cfg(feature = "gamepad")]
            gamepad_hotkeys,
        } = self;
        Menu {
            items,
            selected_index,
            hotkeys,
            #[cfg(feature = "gamepad")]
            gamepad_hotkeys,
        }
    }
}

impl<T: Clone, S> Component for Menu<T, S> {
    type State = S;
    type Output = Option<T>;
    fn render(&self, state: &S, ctx: Ctx, fb: &mut FrameBuffer) {
        for (i, item) in self.items.iter().enumerate() {
            match item {
                MenuItem::Choice(choice) => {
                    let identifier = if i == self.selected_index {
                        &choice.identifier_selected
                    } else {
                        &choice.identifier_deselected
                    };
                    identifier.render(
                        state,
                        ctx.add_offset(Coord::new(0, i as i32))
                            .set_size(Size::new(ctx.bounding_box.size().width(), 1)),
                        fb,
                    );
                }
                MenuItem::Space => (),
            }
        }
    }
    fn update(&mut self, state: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        None
    }
}
