use prototty_input::{keys, Input, KeyboardInput, MouseInput, ScrollDirection};
use prototty_render::Coord;
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

#[derive(Debug, Clone)]
pub enum MenuOutput<T> {
    Quit,
    Cancel,
    Finalise(T),
}

#[derive(Debug, Clone)]
pub struct Escape;

#[derive(Debug, Clone)]
pub struct Quit;

#[derive(Debug, Clone)]
pub enum Cancel {
    Escape,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub struct InitialIndexOutOfBounds;

pub trait MenuIndexFromScreenCoord {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize>;
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

    pub fn selected(&self) -> T {
        self.items[self.selected_index].clone()
    }

    pub fn choose<M>(&mut self, view: &M, input: Input) -> Option<T>
    where
        M: MenuIndexFromScreenCoord,
    {
        match input {
            Input::Keyboard(keys::RETURN) => {
                return Some(self.selected());
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
                    return Some(self.selected());
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
