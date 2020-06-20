use chargrid_input::{keys, Input, KeyboardInput, MouseInput, ScrollDirection};
use chargrid_render::Coord;
use chargrid_render::Size;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MenuInstance<T: Clone> {
    pub(crate) items: Vec<T>,
    pub(crate) selected_index: usize,
    pub(crate) hotkeys: HashMap<char, T>,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuOutput<T> {
    Quit,
    Cancel,
    Finalise(T),
}

#[derive(Debug, Clone, Copy)]
pub struct Escape;

#[derive(Debug, Clone, Copy)]
pub struct Quit;

#[derive(Debug, Clone, Copy)]
pub enum Cancel {
    Escape,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub struct InitialIndexOutOfBounds;

#[derive(Debug, Clone, Copy)]
pub struct Selected;

pub trait MenuIndexFromScreenCoord {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize>;
}

#[derive(Debug)]
pub struct MenuInstanceMouseTracker {
    last_offset: Coord,
    last_size: Size,
}

impl Default for MenuInstanceMouseTracker {
    fn default() -> Self {
        Self {
            last_offset: Coord::new(0, 0),
            last_size: Size::new_u16(0, 0),
        }
    }
}

impl MenuInstanceMouseTracker {
    pub fn new_frame(&mut self, context_offset: Coord) {
        self.last_offset = context_offset;
        self.last_size = Size::new_u16(0, 0);
    }
    pub fn on_entry_view_size(&mut self, size: Size) {
        self.last_size
            .set_width_in_place(self.last_size.width().max(size.width()));
        self.last_size
            .set_height_in_place(self.last_size.height() + size.height());
    }
    pub fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        let rel_coord = coord - self.last_offset;
        if rel_coord.x < 0 || rel_coord.y < 0 || rel_coord.x >= self.last_size.x() as i32 || rel_coord.y >= len as i32 {
            None
        } else {
            Some(rel_coord.y as usize)
        }
    }
}

pub struct MenuInstanceBuilder<T: Clone> {
    pub items: Vec<T>,
    pub selected_index: usize,
    pub hotkeys: Option<HashMap<char, T>>,
}

impl<T: Clone> MenuInstanceBuilder<T> {
    pub fn build(self) -> Result<MenuInstance<T>, InitialIndexOutOfBounds> {
        let Self {
            items,
            selected_index,
            hotkeys,
        } = self;
        if selected_index >= items.len() {
            return Err(InitialIndexOutOfBounds);
        }
        Ok(MenuInstance {
            items,
            selected_index,
            hotkeys: hotkeys.unwrap_or_default(),
        })
    }
}

impl<T: Clone> MenuInstance<T> {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn new(items: Vec<T>) -> Result<Self, InitialIndexOutOfBounds> {
        MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: None,
        }
        .build()
    }

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
        &self.items[self.selected_index]
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (usize, &T, Option<Selected>)> {
        let selected_index = self.selected_index;
        self.items.iter().enumerate().map(move |(i, item)| {
            let selected = if i == selected_index { Some(Selected) } else { None };
            (i, item, selected)
        })
    }

    pub fn choose<M>(&mut self, view: &M, input: Input) -> Option<T>
    where
        M: MenuIndexFromScreenCoord,
    {
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
            Input::Keyboard(KeyboardInput::Char(c)) => {
                if let Some(item) = self.hotkeys.get(&c).cloned() {
                    return Some(item);
                }
            }
            Input::Mouse(MouseInput::MouseMove { coord, .. }) => {
                if let Some(index) = view.menu_index_from_screen_coord(self.items.len(), coord) {
                    self.set_index(index);
                }
            }
            Input::Mouse(MouseInput::MousePress { coord, .. }) => {
                if let Some(index) = view.menu_index_from_screen_coord(self.items.len(), coord) {
                    self.set_index(index);
                    return Some(self.selected().clone());
                }
            }
            #[cfg(feature = "gamepad")]
            Input::Gamepad(gamepad_input) => {
                use chargrid_input::GamepadButton;
                match gamepad_input.button {
                    GamepadButton::DPadDown => self.down(),
                    GamepadButton::DPadUp => self.up(),
                    GamepadButton::Start | GamepadButton::South => return Some(self.selected().clone()),
                    _ => (),
                }
            }
            _ => (),
        }
        None
    }

    pub fn choose_or_escape<M>(&mut self, view: &M, input: Input) -> Option<Result<T, Escape>>
    where
        M: MenuIndexFromScreenCoord,
    {
        match input {
            Input::Keyboard(keys::ESCAPE) => Some(Err(Escape)),
            #[cfg(feature = "gamepad")]
            Input::Gamepad(chargrid_input::GamepadInput {
                button: chargrid_input::GamepadButton::East,
                ..
            }) => Some(Err(Escape)),
            other => self.choose(view, other).map(Ok),
        }
    }

    pub fn choose_or_quit<M>(&mut self, view: &M, input: Input) -> Option<Result<T, Quit>>
    where
        M: MenuIndexFromScreenCoord,
    {
        match input {
            Input::Keyboard(keys::ETX) => Some(Err(Quit)),
            other => self.choose(view, other).map(Ok),
        }
    }

    pub fn choose_or_cancel<M>(&mut self, view: &M, input: Input) -> Option<Result<T, Cancel>>
    where
        M: MenuIndexFromScreenCoord,
    {
        match input {
            Input::Keyboard(keys::ESCAPE) => Some(Err(Cancel::Escape)),
            Input::Keyboard(keys::ETX) => Some(Err(Cancel::Quit)),
            other => self.choose(view, other).map(Ok),
        }
    }

    pub fn into_just_choose(self) -> MenuInstanceJustChoose<T> {
        MenuInstanceJustChoose(self)
    }
    pub fn into_choose_or_escape(self) -> MenuInstanceChooseOrEscape<T> {
        MenuInstanceChooseOrEscape(self)
    }
    pub fn into_choose_or_quit(self) -> MenuInstanceChooseOrQuit<T> {
        MenuInstanceChooseOrQuit(self)
    }
    pub fn into_choose_or_cancel(self) -> MenuInstanceChooseOrCancel<T> {
        MenuInstanceChooseOrCancel(self)
    }
}

pub trait MenuInstanceChoose {
    type Entry: Clone;
    type Output;
    fn choose<M>(&mut self, view: &M, input: Input) -> Option<Self::Output>
    where
        M: MenuIndexFromScreenCoord;
    fn menu_instance(&self) -> &MenuInstance<Self::Entry>;
    fn menu_instance_mut(&mut self) -> &mut MenuInstance<Self::Entry>;
}

pub struct MenuInstanceJustChoose<T: Clone>(MenuInstance<T>);
impl<T: Clone> MenuInstanceChoose for MenuInstanceJustChoose<T> {
    type Entry = T;
    type Output = T;
    fn choose<M>(&mut self, view: &M, input: Input) -> Option<Self::Output>
    where
        M: MenuIndexFromScreenCoord,
    {
        self.0.choose(view, input)
    }
    fn menu_instance(&self) -> &MenuInstance<Self::Entry> {
        &self.0
    }
    fn menu_instance_mut(&mut self) -> &mut MenuInstance<Self::Entry> {
        &mut self.0
    }
}

pub struct MenuInstanceChooseOrEscape<T: Clone>(MenuInstance<T>);
impl<T: Clone> MenuInstanceChoose for MenuInstanceChooseOrEscape<T> {
    type Entry = T;
    type Output = Result<T, Escape>;
    fn choose<M>(&mut self, view: &M, input: Input) -> Option<Self::Output>
    where
        M: MenuIndexFromScreenCoord,
    {
        self.0.choose_or_escape(view, input)
    }
    fn menu_instance(&self) -> &MenuInstance<Self::Entry> {
        &self.0
    }
    fn menu_instance_mut(&mut self) -> &mut MenuInstance<Self::Entry> {
        &mut self.0
    }
}

pub struct MenuInstanceChooseOrQuit<T: Clone>(MenuInstance<T>);
impl<T: Clone> MenuInstanceChoose for MenuInstanceChooseOrQuit<T> {
    type Entry = T;
    type Output = Result<T, Quit>;
    fn choose<M>(&mut self, view: &M, input: Input) -> Option<Self::Output>
    where
        M: MenuIndexFromScreenCoord,
    {
        self.0.choose_or_quit(view, input)
    }
    fn menu_instance(&self) -> &MenuInstance<Self::Entry> {
        &self.0
    }
    fn menu_instance_mut(&mut self) -> &mut MenuInstance<Self::Entry> {
        &mut self.0
    }
}

pub struct MenuInstanceChooseOrCancel<T: Clone>(MenuInstance<T>);
impl<T: Clone> MenuInstanceChoose for MenuInstanceChooseOrCancel<T> {
    type Entry = T;
    type Output = Result<T, Cancel>;
    fn choose<M>(&mut self, view: &M, input: Input) -> Option<Self::Output>
    where
        M: MenuIndexFromScreenCoord,
    {
        self.0.choose_or_cancel(view, input)
    }
    fn menu_instance(&self) -> &MenuInstance<Self::Entry> {
        &self.0
    }
    fn menu_instance_mut(&mut self) -> &mut MenuInstance<Self::Entry> {
        &mut self.0
    }
}
