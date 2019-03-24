extern crate prototty_input;
extern crate prototty_render;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

use prototty_input::{inputs, Input, ScrollDirection};
use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MenuInstance<T> {
    menu: Vec<T>,
    selected_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuOutput<'a, T> {
    Quit,
    Cancel,
    Finalise(&'a T),
}

#[derive(Debug, Clone, Copy)]
pub struct InitialIndexOutOfBounds;

pub trait MenuIndexFromScreenCoord {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize>;
}

impl<T> MenuInstance<T> {
    pub fn new_with_selection(
        menu: Vec<T>,
        selected_index: usize,
    ) -> Result<Self, InitialIndexOutOfBounds> {
        if selected_index >= menu.len() {
            Err(InitialIndexOutOfBounds)
        } else {
            Ok(Self {
                menu,
                selected_index,
            })
        }
    }

    pub fn new(menu: Vec<T>) -> Result<Self, InitialIndexOutOfBounds> {
        Self::new_with_selection(menu, 0)
    }

    pub fn up(&mut self) {
        match self.selected_index.checked_sub(1) {
            Some(index) => self.selected_index = index,
            None => self.selected_index = self.menu.len() - 1,
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < self.menu.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.menu.len() {
            self.selected_index = index;
        }
    }

    pub fn selected(&self) -> &T {
        &self.menu[self.selected_index]
    }

    pub fn tick<I>(&mut self, inputs: I) -> Option<MenuOutput<T>>
    where
        I: IntoIterator<Item = Input>,
    {
        for input in inputs {
            match input {
                inputs::ETX => return Some(MenuOutput::Quit),
                inputs::ESCAPE => return Some(MenuOutput::Cancel),
                inputs::RETURN => {
                    return Some(MenuOutput::Finalise(self.selected()));
                }
                Input::Up => self.up(),
                Input::Down => self.down(),
                _ => (),
            }
        }
        None
    }

    pub fn tick_with_mouse<'a, I, M>(
        &mut self,
        inputs: I,
        view: &'a M,
    ) -> Option<MenuOutput<T>>
    where
        I: IntoIterator<Item = Input>,
        M: MenuIndexFromScreenCoord,
    {
        for input in inputs {
            match input {
                inputs::ETX => return Some(MenuOutput::Quit),
                inputs::ESCAPE => return Some(MenuOutput::Cancel),
                inputs::RETURN => {
                    return Some(MenuOutput::Finalise(self.selected()));
                }
                Input::Up
                | Input::MouseScroll {
                    direction: ScrollDirection::Up,
                    ..
                } => self.up(),
                Input::Down
                | Input::MouseScroll {
                    direction: ScrollDirection::Down,
                    ..
                } => self.down(),
                Input::MouseMove { coord, .. } => {
                    if let Some(index) =
                        view.menu_index_from_screen_coord(self.menu.len(), coord)
                    {
                        self.set_index(index);
                    }
                }
                Input::MousePress { coord, .. } => {
                    if let Some(index) =
                        view.menu_index_from_screen_coord(self.menu.len(), coord)
                    {
                        self.set_index(index);
                        return Some(MenuOutput::Finalise(self.selected()));
                    }
                }
                _ => (),
            }
        }
        None
    }
}

pub struct MenuInstanceView<E> {
    pub entry_view: E,
    last_offset: Coord,
    last_size: Size,
}

impl<E> MenuInstanceView<E> {
    pub fn new(entry_view: E) -> Self {
        Self {
            entry_view,
            last_offset: Coord::new(0, 0),
            last_size: Size::new_u16(0, 0),
        }
    }
}

impl<E> MenuIndexFromScreenCoord for MenuInstanceView<E> {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        let rel_coord = coord - self.last_offset;
        if rel_coord.x < 0
            || rel_coord.y < 0
            || rel_coord.x >= self.last_size.x() as i32
            || rel_coord.y >= len as i32
        {
            None
        } else {
            Some(rel_coord.y as usize)
        }
    }
}

impl<'a, T, E> View<&'a MenuInstance<T>> for MenuInstanceView<E>
where
    E: MenuEntryView<T>,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        menu_instance: &'a MenuInstance<T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.last_offset = context.outer_offset;
        let mut max_width = 0;
        for (i, entry) in menu_instance.menu.iter().enumerate() {
            let width = if i == menu_instance.selected_index {
                self.entry_view.selected(
                    entry,
                    context.add_offset(Coord::new(0, i as i32)),
                    grid,
                )
            } else {
                self.entry_view.normal(
                    entry,
                    context.add_offset(Coord::new(0, i as i32)),
                    grid,
                )
            };
            max_width = max_width.max(width);
        }
        self.last_size = Size::new(max_width, menu_instance.menu.len() as u32);
    }
}

impl<'a, L, T, E> View<(&'a MenuInstance<T>, &'a L)> for MenuInstanceView<E>
where
    E: MenuEntryLookupView<T, L>,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (menu_instance, lookup): (&'a MenuInstance<T>, &'a L),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.last_offset = context.outer_offset;
        let mut max_width = 0;
        for (i, entry) in menu_instance.menu.iter().enumerate() {
            let width = if i == menu_instance.selected_index {
                self.entry_view.selected(
                    entry,
                    lookup,
                    context.add_offset(Coord::new(0, i as i32)),
                    grid,
                )
            } else {
                self.entry_view.normal(
                    entry,
                    lookup,
                    context.add_offset(Coord::new(0, i as i32)),
                    grid,
                )
            };
            max_width = max_width.max(width);
        }
        self.last_size = Size::new(max_width, menu_instance.menu.len() as u32);
    }
}

pub trait MenuEntryView<T> {
    fn normal<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32;
    fn selected<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32;
}

pub trait MenuEntryLookupView<T, L> {
    fn normal<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        lookup: &L,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32;
    fn selected<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        lookup: &L,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32;
}
