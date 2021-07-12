use chargrid_component::*;
use std::collections::HashMap;
use std::time::Duration;

pub trait MenuItemIdentifier: Component {
    fn init_selection(&mut self, selection: bool);
    fn set_selection(&mut self, selection: bool);
}

pub type MenuItemIdentifierBoxed<S = ()> = Box<dyn MenuItemIdentifier<State = S, Output = ()>>;

pub struct MenuItem<T: Clone, S = ()> {
    value: T,
    identifier: MenuItemIdentifierBoxed<S>,
}

pub struct Menu<T: Clone, S = ()> {
    items: Vec<MenuItem<T, S>>,
    offset_to_item_index: Vec<Option<usize>>,
    selected_index: usize,
    hotkeys: HashMap<input::KeyboardInput, T>,
    since_epoch: Duration,
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
            Some(index) => self.set_index(index),
            None => self.set_index(self.items.len() - 1),
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.set_index(self.selected_index + 1);
        } else {
            self.set_index(0);
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.items.len() {
            if self.selected_index != index {
                self.items[self.selected_index]
                    .identifier
                    .set_selection(false);
                self.selected_index = index;
                self.items[self.selected_index]
                    .identifier
                    .set_selection(true);
            }
        }
    }

    pub fn index(&self) -> usize {
        self.selected_index
    }

    pub fn selected(&self) -> &T {
        &self.items[self.selected_index].value
    }

    fn menu_index_from_screen_coord(&self, ctx: Ctx, coord: Coord) -> Option<usize> {
        if let Some(relative_coord) = ctx.bounding_box.coord_absolute_to_relative(coord) {
            let index = relative_coord.y as usize;
            if let Some(&Some(item_index)) = self.offset_to_item_index.get(index) {
                Some(item_index)
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
        for (offset, &item_index) in self.offset_to_item_index.iter().enumerate() {
            if let Some(item_index) = item_index {
                self.items[item_index].identifier.render(
                    state,
                    ctx.add_offset(Coord::new(0, offset as i32))
                        .set_size(Size::new(ctx.bounding_box.size().width(), 1)),
                    fb,
                );
            }
        }
    }
    fn update(&mut self, state: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input) => self.choose(ctx, input),
            Event::Tick(duration) => {
                self.since_epoch += duration;
                for item in self.items.iter_mut() {
                    item.identifier.update(state, ctx, event);
                }
                None
            }
            _ => None,
        }
    }
}

pub mod identifier {
    use super::*;
    use crate::text::{StyledString, Text};

    pub struct MenuItemIdentifierStaticInner {
        selected: Text,
        deselected: Text,
        is_selected: bool,
    }

    impl PureStaticComponent for MenuItemIdentifierStaticInner {
        fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
            if self.is_selected {
                self.selected.render(ctx, fb);
            } else {
                self.deselected.render(ctx, fb);
            }
        }
    }

    pub type MenuItemIdentifierStatic =
        convert::PureStaticComponentT<MenuItemIdentifierStaticInner>;

    impl MenuItemIdentifier for MenuItemIdentifierStatic {
        fn set_selection(&mut self, selection: bool) {
            self.0.is_selected = selection;
        }
        fn init_selection(&mut self, selection: bool) {
            self.0.is_selected = selection;
        }
    }

    pub fn static_<S: Into<Text>, D: Into<Text>>(
        selected: S,
        deselected: D,
    ) -> MenuItemIdentifierBoxed {
        Box::new(
            MenuItemIdentifierStaticInner {
                selected: selected.into(),
                deselected: deselected.into(),
                is_selected: false,
            }
            .component(),
        )
    }

    pub struct MenuItemIdentifierDynamicCtx<'a> {
        pub component: &'a mut Text,
        pub since_change: Duration,
        pub is_selected: bool,
        pub styles_prev: &'a [Style],
    }

    pub trait MenuItemIdentifierDynamicUpdate {
        fn update(&mut self, ctx: MenuItemIdentifierDynamicCtx);
    }

    impl<F: FnMut(MenuItemIdentifierDynamicCtx)> MenuItemIdentifierDynamicUpdate for F {
        fn update(&mut self, ctx: MenuItemIdentifierDynamicCtx) {
            (self)(ctx);
        }
    }

    pub struct MenuItemIdentifierDynamicInner<U: MenuItemIdentifierDynamicUpdate> {
        update: U,
        component: Text,
        since_change: Duration,
        is_selected: bool,
        styles_prev: Vec<Style>,
    }

    impl<U: MenuItemIdentifierDynamicUpdate> PureComponent for MenuItemIdentifierDynamicInner<U> {
        type Output = ();
        fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
            self.component.render(ctx, fb);
        }
        fn update(&mut self, _ctx: Ctx, event: Event) -> Self::Output {
            if let Some(duration) = event.tick() {
                self.since_change += duration;
                for part in self.component.parts.iter_mut() {
                    part.string.clear();
                }
                self.update.update(MenuItemIdentifierDynamicCtx {
                    component: &mut self.component,
                    since_change: self.since_change,
                    is_selected: self.is_selected,
                    styles_prev: &self.styles_prev,
                });
            }
        }
    }

    const LONG_DURATION: Duration = Duration::from_secs(31536000); // 1 year

    pub type MenuItemIdentifierDynamic<U> =
        convert::PureComponentT<MenuItemIdentifierDynamicInner<U>>;

    impl<U: MenuItemIdentifierDynamicUpdate> MenuItemIdentifier for MenuItemIdentifierDynamic<U> {
        fn init_selection(&mut self, selection: bool) {
            self.0.is_selected = selection;
            self.0.since_change = LONG_DURATION;
        }
        fn set_selection(&mut self, selection: bool) {
            self.0.is_selected = selection;
            self.0.since_change = Duration::from_secs(0);
            self.0.styles_prev = self.0.component.parts.iter().map(|p| p.style).collect();
        }
    }

    pub fn dynamic<U: 'static + MenuItemIdentifierDynamicUpdate>(
        initial_num_parts: usize,
        update: U,
    ) -> MenuItemIdentifierBoxed {
        let mut parts = Vec::new();
        let mut styles_prev = Vec::new();
        for _ in 0..initial_num_parts {
            parts.push(StyledString {
                string: String::new(),
                style: Style::default(),
            });
            styles_prev.push(Style::default());
        }
        let component = Text::new(parts);
        Box::new(
            MenuItemIdentifierDynamicInner {
                update,
                component,
                since_change: LONG_DURATION,
                is_selected: false,
                styles_prev,
            }
            .component(),
        )
    }

    pub fn dynamic_fn<F: 'static + FnMut(MenuItemIdentifierDynamicCtx)>(
        initial_num_parts: usize,
        f: F,
    ) -> MenuItemIdentifierBoxed {
        dynamic(initial_num_parts, f)
    }
}

pub mod builder {
    use super::*;

    pub use super::identifier;
    pub use crate::text::StyledString;
    pub use chargrid_component::Rgba32;
    pub use input::KeyboardInput;
    pub use std::fmt::Write;

    pub struct MenuBuilderAddItem<T: Clone, S = ()> {
        pub value: T,
        pub identifier: MenuItemIdentifierBoxed<S>,
        pub hotkeys: Vec<input::KeyboardInput>,
    }

    impl<T: Clone, S> MenuBuilderAddItem<T, S> {
        pub fn new(value: T, identifier: MenuItemIdentifierBoxed<S>) -> Self {
            Self {
                value,
                identifier,
                hotkeys: Vec::new(),
            }
        }

        pub fn add_hotkey(mut self, input: input::KeyboardInput) -> Self {
            self.hotkeys.push(input);
            self
        }

        pub fn add_hotkey_char(mut self, c: char) -> Self {
            self.hotkeys.push(KeyboardInput::Char(c));
            self
        }
    }

    pub struct MenuBuilder<T: Clone, S = ()> {
        items: Vec<MenuItem<T, S>>,
        hotkeys: HashMap<input::KeyboardInput, T>,
        offset_to_item_index: Vec<Option<usize>>,
    }

    impl<T: Clone, S> Default for MenuBuilder<T, S> {
        fn default() -> Self {
            Self {
                items: Vec::new(),
                offset_to_item_index: Vec::new(),
                hotkeys: HashMap::new(),
            }
        }
    }

    impl<T: Clone, S> MenuBuilder<T, S> {
        pub fn add_item(mut self, add_item: MenuBuilderAddItem<T, S>) -> Self {
            for hotkey in add_item.hotkeys {
                if self
                    .hotkeys
                    .insert(hotkey, add_item.value.clone())
                    .is_some()
                {
                    panic!("Duplicate hotkey: {:?}", hotkey);
                }
            }
            self.offset_to_item_index.push(Some(self.items.len()));
            self.items.push(MenuItem {
                identifier: add_item.identifier,
                value: add_item.value,
            });
            self
        }

        pub fn add_space(mut self) -> Self {
            self.offset_to_item_index.push(None);
            self
        }

        pub fn build(self) -> Menu<T, S> {
            let Self {
                mut items,
                hotkeys,
                offset_to_item_index,
            } = self;
            let selected_index = 0;
            for (i, item) in items.iter_mut().enumerate() {
                if i == selected_index {
                    item.identifier.init_selection(true);
                } else {
                    item.identifier.init_selection(false);
                }
            }
            Menu {
                items,
                selected_index,
                offset_to_item_index,
                hotkeys,
                since_epoch: Duration::from_millis(0),
            }
        }
    }

    pub fn menu_builder<T: Clone, S>() -> MenuBuilder<T, S> {
        MenuBuilder::default()
    }

    pub fn item<T: Clone, S>(
        value: T,
        identifier: MenuItemIdentifierBoxed<S>,
    ) -> MenuBuilderAddItem<T, S> {
        MenuBuilderAddItem::new(value, identifier)
    }
}
