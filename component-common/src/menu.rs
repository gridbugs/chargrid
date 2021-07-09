use chargrid_component::*;
use std::collections::HashMap;

pub struct MenuItemChoiceIdentifiers<S = ()> {
    pub selected: Box<dyn Component<State = S, Output = ()>>,
    pub deselected: Box<dyn Component<State = S, Output = ()>>,
}

pub struct MenuItemChoice<T: Clone, S = ()> {
    pub value: T,
    pub identifiers: MenuItemChoiceIdentifiers<S>,
}

pub enum MenuItem<T: Clone, S = ()> {
    Choice(MenuItemChoice<T, S>),
}

pub struct Menu<T: Clone, S = ()> {
    items: Vec<MenuItem<T, S>>,
    selected_index: usize,
    hotkeys: HashMap<input::KeyboardInput, T>,
    #[cfg(feature = "gamepad")]
    gamepad_hotkeys: HashMap<input::GamepadButton, T>,
}

pub type PureMenu<T> = convert::ComponentPureT<Menu<T, ()>>;

impl<T: Clone> Menu<T, ()> {
    pub fn pure(self) -> PureMenu<T> {
        convert::ComponentPureT(self)
    }
}

impl<T: Clone, S> Menu<T, S> {
    pub fn up(&mut self) {
        match self.selected_index.checked_sub(1) {
            Some(index) => self.selected_index = index,
            None => self.selected_index = self.items.len() - 1,
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }

    pub fn index(&self) -> usize {
        self.selected_index
    }

    pub fn selected(&self) -> &T {
        match &self.items[self.selected_index] {
            MenuItem::Choice(choice) => &choice.value,
        }
    }

    fn menu_index_from_screen_coord(&self, ctx: Ctx, coord: Coord) -> Option<usize> {
        if let Some(relative_coord) = ctx.bounding_box.coord_absolute_to_relative(coord) {
            let index = relative_coord.y as usize;
            if index < self.items.len() {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn choose(&mut self, ctx: Ctx, input: input::Input) -> Option<T> {
        use input::*;
        match input {
            Input::Keyboard(keys::RETURN) | Input::Keyboard(KeyboardInput::Char(' ')) => {
                return Some(self.selected().clone());
            }
            Input::Keyboard(KeyboardInput::Up)
            | Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Up,
                ..
            }) => self.up(),
            Input::Keyboard(KeyboardInput::Down)
            | Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Down,
                ..
            }) => self.down(),
            Input::Keyboard(other) => {
                if let Some(item) = self.hotkeys.get(&other).cloned() {
                    return Some(item);
                }
            }
            Input::Mouse(MouseInput::MouseMove { coord, .. }) => {
                if let Some(index) = self.menu_index_from_screen_coord(ctx, coord) {
                    self.set_index(index);
                }
            }
            Input::Mouse(MouseInput::MousePress { coord, .. }) => {
                if let Some(index) = self.menu_index_from_screen_coord(ctx, coord) {
                    self.set_index(index);
                    return Some(self.selected().clone());
                }
            }
            #[cfg(feature = "gamepad")]
            Input::Gamepad(gamepad_input) => match gamepad_input.button {
                GamepadButton::DPadDown => self.down(),
                GamepadButton::DPadUp => self.up(),
                GamepadButton::Start | GamepadButton::South => {
                    return Some(self.selected().clone())
                }
                _ => (),
            },
            _ => (),
        }
        None
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
                        &choice.identifiers.selected
                    } else {
                        &choice.identifiers.deselected
                    };
                    identifier.render(
                        state,
                        ctx.add_offset(Coord::new(0, i as i32))
                            .set_size(Size::new(ctx.bounding_box.size().width(), 1)),
                        fb,
                    );
                }
            }
        }
    }
    fn update(&mut self, _: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        event.input().and_then(|i| self.choose(ctx, i))
    }
}

pub mod builder {
    use super::*;
    use crate::text::StyledString;

    pub use chargrid_component::Rgba32;
    #[cfg(feature = "gamepad")]
    pub use input::GamepadButton;
    pub use input::KeyboardInput;

    pub struct MenuBuilderAddChoice<T: Clone, S = ()> {
        pub value: T,
        pub identifiers: MenuItemChoiceIdentifiers<S>,
        pub hotkeys: Vec<input::KeyboardInput>,
        #[cfg(feature = "gamepad")]
        pub gamepad_hotkeys: Vec<input::GamepadButton>,
    }

    impl<T: Clone, S> MenuBuilderAddChoice<T, S> {
        pub fn new(value: T, identifiers: MenuItemChoiceIdentifiers<S>) -> Self {
            Self {
                value,
                identifiers,
                hotkeys: Vec::new(),
                #[cfg(feature = "gamepad")]
                gamepad_hotkeys: Vec::new(),
            }
        }

        pub fn add_hotkey(mut self, input: input::KeyboardInput) -> Self {
            self.hotkeys.push(input);
            self
        }

        #[cfg(feature = "gamepad")]
        pub fn add_gamepad_hotkey(mut self, input: input::GamepadButton) -> Self {
            self.gamepad_hotkeys.push(input);
            self
        }
    }

    pub struct MenuBuilder<T: Clone, S = ()> {
        pub items: Vec<MenuItem<T, S>>,
        pub hotkeys: HashMap<input::KeyboardInput, T>,
        #[cfg(feature = "gamepad")]
        pub gamepad_hotkeys: HashMap<input::GamepadButton, T>,
    }

    impl<T: Clone, S> Default for MenuBuilder<T, S> {
        fn default() -> Self {
            Self {
                items: Vec::new(),
                hotkeys: HashMap::new(),
                #[cfg(feature = "gamepad")]
                gamepad_hotkeys: HashMap::new(),
            }
        }
    }

    impl<T: Clone, S> MenuBuilder<T, S> {
        pub fn add_choice(mut self, add_choice: MenuBuilderAddChoice<T, S>) -> Self {
            for hotkey in add_choice.hotkeys {
                if self
                    .hotkeys
                    .insert(hotkey, add_choice.value.clone())
                    .is_some()
                {
                    panic!("Duplicate hotkey: {:?}", hotkey);
                }
            }
            #[cfg(feature = "gamepad")]
            for hotkey in add_choice.gamepad_hotkeys {
                if self
                    .gamepad_hotkeys
                    .insert(hotkey, add_choice.value.clone())
                    .is_some()
                {
                    panic!("Duplicate hotkey: {:?}", hotkey);
                }
            }
            self.items.push(MenuItem::Choice(MenuItemChoice {
                identifiers: add_choice.identifiers,
                value: add_choice.value,
            }));
            self
        }

        pub fn build(self) -> Menu<T, S> {
            let Self {
                items,
                hotkeys,
                #[cfg(feature = "gamepad")]
                gamepad_hotkeys,
            } = self;
            Menu {
                items,
                selected_index: 0,
                hotkeys,
                #[cfg(feature = "gamepad")]
                gamepad_hotkeys,
            }
        }
    }

    pub fn menu_builder<T: Clone, S>() -> MenuBuilder<T, S> {
        MenuBuilder::default()
    }

    pub fn choice<T: Clone, S>(
        value: T,
        identifiers: MenuItemChoiceIdentifiers<S>,
    ) -> MenuBuilderAddChoice<T, S> {
        MenuBuilderAddChoice::new(value, identifiers)
    }

    pub fn identifiers<S>(
        selected: Box<dyn Component<State = S, Output = ()>>,
        deselected: Box<dyn Component<State = S, Output = ()>>,
    ) -> MenuItemChoiceIdentifiers<S> {
        MenuItemChoiceIdentifiers {
            selected,
            deselected,
        }
    }

    pub fn identifiers_styled_strings<S: AsRef<str>, D: AsRef<str>>(
        selected_string: S,
        deselected_string: D,
        selected_style: Style,
        deselected_style: Style,
    ) -> MenuItemChoiceIdentifiers<()> {
        let selected = StyledString {
            string: selected_string.as_ref().to_string(),
            style: selected_style,
        };
        let deselected = StyledString {
            string: deselected_string.as_ref().to_string(),
            style: deselected_style,
        };
        identifiers(
            Box::new(selected.component()),
            Box::new(deselected.component()),
        )
    }

    pub fn identifiers_bold_selected<S: AsRef<str>>(string: S) -> MenuItemChoiceIdentifiers<()> {
        identifiers_styled_strings(
            string.as_ref(),
            string.as_ref(),
            Style {
                bold: Some(true),
                ..Style::default()
            },
            Style {
                bold: Some(false),
                ..Style::default()
            },
        )
    }
}
