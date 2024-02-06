use crate::control_flow::*;
use chargrid_core::*;
use std::collections::HashMap;
use std::time::Duration;

pub trait MenuItemIdentifier: Component {
    fn init_selection(&mut self, selection: bool);
    fn set_selection(&mut self, selection: bool);
}

pub type MenuItemIdentifierBoxed<S = ()> = Box<dyn MenuItemIdentifier<State = S, Output = ()>>;
pub type MenuCF<T, S> = CF<Option<T>, S>;

pub struct MenuItem<T: Clone, S = ()> {
    value: T,
    identifier: MenuItemIdentifierBoxed<S>,
}

pub struct Menu<T: Clone, S = ()> {
    items: Vec<MenuItem<T, S>>,
    offset_to_item_index: Vec<Option<usize>>,
    selected_index: usize,
    hotkeys: HashMap<input::KeyboardInput, T>,
    vi_keys: bool,
    since_epoch: Duration,
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
            Input::Keyboard(keyboard_input) => match keyboard_input {
                keys::RETURN | KeyboardInput::Char(' ') => {
                    return Some(self.selected().clone());
                }
                KeyboardInput::Up => self.up(),
                KeyboardInput::Down => self.down(),
                other => {
                    if self.vi_keys {
                        if other == KeyboardInput::Char('j') {
                            self.down();
                            return None;
                        }
                        if other == KeyboardInput::Char('k') {
                            self.up();
                            return None;
                        }
                    }
                    if let Some(item) = self.hotkeys.get(&other).cloned() {
                        return Some(item);
                    }
                }
            },
            Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Up,
                ..
            }) => self.up(),
            Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Down,
                ..
            }) => self.down(),
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

    fn size(&self, state: &S, ctx: Ctx) -> Size {
        let mut max_coord = Coord::new(0, 0);
        for (offset, &item_index) in self.offset_to_item_index.iter().enumerate() {
            if let Some(item_index) = item_index {
                let offset = Coord::new(0, offset as i32);
                let identifier_size = self.items[item_index].identifier.size(
                    state,
                    ctx.add_offset(offset)
                        .set_size(Size::new(ctx.bounding_box.size().width(), 1)),
                );
                let bottom_right = offset + identifier_size.to_coord().unwrap();
                max_coord.x = max_coord.x.max(bottom_right.x);
                max_coord.y = max_coord.y.max(bottom_right.y);
            }
        }
        max_coord.to_size().unwrap()
    }
}

pub mod identifier {
    use super::*;
    use crate::text::{StyledString, Text};

    pub struct MenuItemIdentifierStatic {
        selected: Text,
        deselected: Text,
        is_selected: bool,
    }

    impl Component for MenuItemIdentifierStatic {
        type Output = ();
        type State = ();
        fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
            if self.is_selected {
                self.selected.render(state, ctx, fb);
            } else {
                self.deselected.render(state, ctx, fb);
            }
        }
        fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {}
        fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
            if self.is_selected {
                self.selected.size(state, ctx)
            } else {
                self.deselected.size(state, ctx)
            }
        }
    }

    impl MenuItemIdentifier for MenuItemIdentifierStatic {
        fn set_selection(&mut self, selection: bool) {
            self.is_selected = selection;
        }
        fn init_selection(&mut self, selection: bool) {
            self.is_selected = selection;
        }
    }

    pub fn static_<S: Into<Text>, D: Into<Text>>(
        selected: S,
        deselected: D,
    ) -> MenuItemIdentifierBoxed {
        Box::new(MenuItemIdentifierStatic {
            selected: selected.into(),
            deselected: deselected.into(),
            is_selected: false,
        })
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

    pub struct MenuItemIdentifierDynamic<U: MenuItemIdentifierDynamicUpdate> {
        update: U,
        component: Text,
        since_change: Duration,
        is_selected: bool,
        styles_prev: Vec<Style>,
    }

    impl<U: MenuItemIdentifierDynamicUpdate> Component for MenuItemIdentifierDynamic<U> {
        type Output = ();
        type State = ();
        fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
            self.component.render(state, ctx, fb);
        }
        fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
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
        fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
            self.component.size(state, ctx)
        }
    }

    const LONG_DURATION: Duration = Duration::from_secs(31536000); // 1 year

    impl<U: MenuItemIdentifierDynamicUpdate> MenuItemIdentifier for MenuItemIdentifierDynamic<U> {
        fn init_selection(&mut self, selection: bool) {
            self.is_selected = selection;
            self.since_change = LONG_DURATION;
        }
        fn set_selection(&mut self, selection: bool) {
            self.is_selected = selection;
            self.since_change = Duration::from_secs(0);
            self.styles_prev = self.component.parts.iter().map(|p| p.style).collect();
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
        Box::new(MenuItemIdentifierDynamic {
            update,
            component,
            since_change: LONG_DURATION,
            is_selected: false,
            styles_prev,
        })
    }

    pub fn dynamic_fn<F: 'static + FnMut(MenuItemIdentifierDynamicCtx)>(
        initial_num_parts: usize,
        f: F,
    ) -> MenuItemIdentifierBoxed {
        dynamic(initial_num_parts, f)
    }

    pub fn simple(s: &str) -> MenuItemIdentifierBoxed {
        static_(
            StyledString {
                string: s.to_string(),
                style: Style {
                    bold: Some(true),
                    foreground: Some(Rgba32::new_grey(255)),
                    ..Default::default()
                },
            },
            StyledString {
                string: s.to_string(),
                style: Style {
                    bold: Some(false),
                    foreground: Some(Rgba32::new_grey(255)),
                    ..Default::default()
                },
            },
        )
    }

    pub mod fade_spec {
        pub use chargrid_core::Rgba32;
        pub use std::time::Duration;

        #[derive(Clone, Copy, Debug)]
        pub struct Layers<T> {
            pub foreground: T,
            pub background: T,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct To {
            pub rgba32: Layers<Rgba32>,
            pub bold: bool,
            pub underline: bool,
        }

        #[derive(Clone, Copy, Debug)]
        pub enum FromStyle {
            Current,
            Rgba32(Rgba32),
        }

        #[derive(Clone, Copy, Debug)]
        pub struct From {
            pub style: Layers<FromStyle>,
        }

        impl From {
            pub const fn current() -> Self {
                Self {
                    style: Layers {
                        foreground: FromStyle::Current,
                        background: FromStyle::Current,
                    },
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Fade {
            pub to: To,
            pub from: From,
            pub durations: Layers<Duration>,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FadeSpec {
            pub on_select: Fade,
            pub on_deselect: Fade,
        }

        impl FadeSpec {
            pub fn identifier<F: 'static + FnMut(&mut String)>(
                self,
                writer: F,
            ) -> super::MenuItemIdentifierBoxed {
                super::fade(self, writer)
            }
        }
    }

    pub fn fade<F: 'static + FnMut(&mut String)>(
        spec: fade_spec::FadeSpec,
        mut writer: F,
    ) -> MenuItemIdentifierBoxed {
        dynamic(1, move |ctx: MenuItemIdentifierDynamicCtx| {
            use crate::fade;
            use fade_spec::FromStyle;
            let part = &mut ctx.component.parts[0];
            writer(&mut part.string);
            let fade = if ctx.is_selected {
                &spec.on_select
            } else {
                &spec.on_deselect
            };
            let from_fg = match fade.from.style.foreground {
                FromStyle::Current => ctx.styles_prev[0]
                    .foreground
                    .unwrap_or(fade.to.rgba32.foreground),
                FromStyle::Rgba32(rgba32) => rgba32,
            };
            let fade_fg = fade::linear(
                from_fg,
                fade.to.rgba32.foreground,
                fade.durations.foreground,
            );
            let from_bg = match fade.from.style.background {
                FromStyle::Current => ctx.styles_prev[0]
                    .background
                    .unwrap_or(fade.to.rgba32.background),
                FromStyle::Rgba32(rgba32) => rgba32,
            };
            let fade_bg = fade::linear(
                from_bg,
                fade.to.rgba32.background,
                fade.durations.background,
            );
            ctx.component.parts[0].style = Style {
                bold: Some(fade.to.bold),
                underline: Some(fade.to.underline),
                foreground: Some(fade_fg.eval(ctx.since_change)),
                background: Some(fade_bg.eval(ctx.since_change)),
            };
        })
    }
}

pub mod builder {
    use super::*;

    pub use super::identifier;
    pub use crate::text::StyledString;
    pub use chargrid_core::Rgba32;
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
            self.hotkeys.push(input::KeyboardInput::Char(c));
            self
        }
    }

    pub struct MenuBuilder<T: Clone, S = ()> {
        items: Vec<MenuItem<T, S>>,
        hotkeys: HashMap<input::KeyboardInput, T>,
        offset_to_item_index: Vec<Option<usize>>,
        vi_keys: bool,
    }

    impl<T: Clone, S> Default for MenuBuilder<T, S> {
        fn default() -> Self {
            Self {
                items: Vec::new(),
                offset_to_item_index: Vec::new(),
                hotkeys: HashMap::new(),
                vi_keys: false,
            }
        }
    }

    impl<T: Clone, S> MenuBuilder<T, S> {
        #[must_use]
        pub fn add_item(mut self, add_item: MenuBuilderAddItem<T, S>) -> Self {
            self.add_item_mut(add_item);
            self
        }

        pub fn add_item_mut(&mut self, add_item: MenuBuilderAddItem<T, S>) {
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
        }

        #[must_use]
        pub fn add_space(mut self) -> Self {
            self.add_space_mut();
            self
        }

        pub fn add_space_mut(&mut self) {
            self.offset_to_item_index.push(None);
        }

        pub fn vi_keys(mut self) -> Self {
            self.vi_keys_mut();
            self
        }

        pub fn vi_keys_mut(&mut self) {
            self.vi_keys = true;
        }

        pub fn build(self) -> Menu<T, S> {
            let Self {
                mut items,
                hotkeys,
                offset_to_item_index,
                vi_keys,
            } = self;
            let selected_index = 0;
            for (i, item) in items.iter_mut().enumerate() {
                if i == selected_index {
                    item.identifier.init_selection(true);
                } else {
                    item.identifier.init_selection(false);
                }
            }
            if vi_keys {
                // make sure the hotkeys don't include the vi keys
                if hotkeys.contains_key(&input::KeyboardInput::Char('j')) {
                    panic!("vi_keys cannot be used when j is a hotkey");
                }
                if hotkeys.contains_key(&input::KeyboardInput::Char('k')) {
                    panic!("vi_keys cannot be used when k is a hotkey");
                }
            }
            Menu {
                items,
                selected_index,
                offset_to_item_index,
                hotkeys,
                vi_keys,
                since_epoch: Duration::from_millis(0),
            }
        }
    }

    impl<T: 'static + Clone> MenuBuilder<T, ()> {
        pub fn build_cf<S: 'static>(self) -> CF<Option<T>, S> {
            cf(self.build()).ignore_state()
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
