use prototty_input::{inputs, Input, ScrollDirection};
use prototty_render::*;
use prototty_text::Style;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MenuEntry<T> {
    pub name: String,
    pub value: T,
}

impl<T, S: Into<String>> From<(S, T)> for MenuEntry<T> {
    fn from((s, t): (S, T)) -> Self {
        Self {
            name: s.into(),
            value: t,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Menu<T> {
    entries: Vec<MenuEntry<T>>,
}

#[derive(Debug, Clone, Copy)]
pub struct InitialIndexOutOfBounds;

impl<T> Menu<T> {
    pub fn new<M, I>(entries: I) -> Self
    where
        M: Into<MenuEntry<T>>,
        I: IntoIterator<Item = M>,
    {
        Self {
            entries: entries.into_iter().map(|m| m.into()).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn instantiate_with_selection(
        self,
        selected_index: usize,
    ) -> Result<MenuInstance<T>, InitialIndexOutOfBounds> {
        MenuInstance::new_with_selection(self, selected_index)
    }

    pub fn instantiate(self) -> Result<MenuInstance<T>, InitialIndexOutOfBounds> {
        MenuInstance::new(self)
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MenuInstance<T> {
    menu: Menu<T>,
    selected_index: usize,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum MenuOutput<'a, T> {
    Quit,
    Cancel,
    Finalise(&'a T),
}

pub trait MenuIndexFromScreenCoord {
    fn menu_index_from_screen_coord<'a, T>(
        &self,
        menu: &'a Menu<T>,
        coord: Coord,
    ) -> Option<usize>;
}

impl<T> MenuInstance<T> {
    pub fn new_with_selection(
        menu: Menu<T>,
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

    pub fn new(menu: Menu<T>) -> Result<Self, InitialIndexOutOfBounds> {
        Self::new_with_selection(menu, 0)
    }

    pub fn up(&mut self) {
        match self.selected_index.checked_sub(1) {
            Some(index) => self.selected_index = index,
            None => self.selected_index = self.menu.entries.len() - 1,
        }
    }

    pub fn down(&mut self) {
        if self.selected_index < self.menu.entries.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.menu.entries.len() {
            self.selected_index = index;
        }
    }

    pub fn selected(&self) -> &T {
        &self.menu.entries[self.selected_index].value
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
                        view.menu_index_from_screen_coord(&self.menu, coord)
                    {
                        self.set_index(index);
                    }
                }
                Input::MousePress { coord, .. } => {
                    if let Some(index) =
                        view.menu_index_from_screen_coord(&self.menu, coord)
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

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct MenuInstanceView {
    pub selected_style: Style,
    pub normal_style: Style,
    last_offset: Coord,
    last_size: Size,
}

impl MenuIndexFromScreenCoord for MenuInstanceView {
    fn menu_index_from_screen_coord<'a, T>(
        &self,
        menu: &'a Menu<T>,
        coord: Coord,
    ) -> Option<usize> {
        let rel_coord = coord - self.last_offset;
        if rel_coord.x < 0
            || rel_coord.y < 0
            || rel_coord.x >= self.last_size.x() as i32
            || rel_coord.y >= menu.entries.len() as i32
        {
            None
        } else {
            Some(rel_coord.y as usize)
        }
    }
}

impl<'a, T> View<&'a MenuInstance<T>> for MenuInstanceView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        value: &'a MenuInstance<T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.last_offset = context.outer_offset;
        let mut max = Coord::new(0, 0);
        for (i, entry) in value.menu.entries.iter().enumerate() {
            let style = if i == value.selected_index {
                self.selected_style
            } else {
                self.normal_style
            };
            for (j, ch) in entry.name.chars().enumerate() {
                let coord = Coord::new(j as i32, i as i32);
                max.x = max.x.max(coord.x);
                max.y = max.y.max(coord.y);
                grid.set_cell_relative(coord, 0, style.view_cell(ch), context);
            }
        }
        self.last_size = max.to_size().unwrap() + Size::new_u16(1, 1);
    }
}
