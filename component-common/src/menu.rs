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
    Space,
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
                MenuItem::Space => (),
            }
        }
    }
    fn update(&mut self, state: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        None
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

        pub fn add_space(mut self) -> Self {
            self.items.push(MenuItem::Space);
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
