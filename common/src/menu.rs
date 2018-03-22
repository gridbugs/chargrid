use std::cmp;
use prototty::*;
use prototty::inputs::*;
use text_info::*;
use defaults::*;

/// A single entry in a menu. It owns the value
/// which will be yielded if this entry is
/// finalised.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuEntry<T: Copy> {
    pub name: String,
    pub value: T,
}

impl<T: Copy, S: Into<String>> From<(S, T)> for MenuEntry<T> {
    fn from((s, t): (S, T)) -> Self {
        Self {
            name: s.into(),
            value: t,
        }
    }
}

/// A list of `MenuEntry`s, in the order they appear when rendered,
/// with a description of how the text of the selected and normal
/// (ie. not selected) entries should be rendered.
///
/// `Menu`s (`MenuEntry`s rather) own their value, and remain in
/// scope after a value has been chosen (by running the menu).
/// A copy of a `MenuEntry`'s value is returned by `MenuRunner::run_menu`.
///
/// Note that a `Menu` doesn't contain information about the current
/// selection. When a `Menu` is rendered, all its entries use
/// `normal_info` when rendering. To combine a `Menu` with selection
/// state, use a `MenuInstance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu<T: Copy> {
    pub entries: Vec<MenuEntry<T>>,
    pub size: Size,
    pub selected_info: TextInfo,
    pub normal_info: TextInfo,
}

fn selected_info() -> TextInfo {
    TextInfo::default()
        .bold()
        .foreground_colour(DEFAULT_BG)
        .background_colour(DEFAULT_FG)
}

impl<T: Copy> Menu<T> {
    /// Create a new menu.
    pub fn new<S, V>(mut e: Vec<(S, T)>, size: V) -> Self
    where
        S: Into<String>,
        V: Into<Size>,
    {
        Self {
            entries: e.drain(..).map(Into::into).collect(),
            size: size.into(),
            normal_info: Default::default(),
            selected_info: selected_info(),
        }
    }

    /// Create a new menu, occupying the smallest amount of
    /// space required to fit all entries.
    pub fn smallest<S>(mut e: Vec<(S, T)>) -> Self
    where
        S: Into<String>,
    {
        let entries: Vec<MenuEntry<T>> = e.drain(..).map(Into::into).collect();
        let width = entries.iter().fold(0, |acc, e| cmp::max(acc, e.name.len()));
        let height = entries.len();
        Self {
            entries,
            size: Size::new(width as u32, height as u32),
            normal_info: Default::default(),
            selected_info: selected_info(),
        }
    }
}

/// The result of a user interacting with a menu.
pub enum MenuOutput<T> {
    Quit,
    Cancel,
    Finalise(T),
}

/// An instance of a menu, with a selected entry.
/// When a `MenuInstance` is rendered, the
/// currently-selected entry is  rendered using
/// the `Menu`'s `selected_info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuInstance<T: Copy> {
    menu: Menu<T>,
    index: usize,
}

impl<T: Copy> MenuInstance<T> {
    /// Create a new `MenuInstance` with the first entry selected.
    /// Returns `None` if the `Menu` has 0 elements.
    pub fn new(menu: Menu<T>) -> Option<Self> {
        Self::with_index(menu, 0)
    }

    /// Create a new `MenuInstance` with the given index selected.
    /// Returns `None` if `index >= menu.entries.len()`.
    pub fn with_index(menu: Menu<T>, index: usize) -> Option<Self> {
        if index < menu.entries.len() {
            Some(Self { menu, index })
        } else {
            None
        }
    }

    /// Returns a reference to the internal menu
    pub fn menu(&self) -> &Menu<T> {
        &self.menu
    }

    /// Consumes the instance, returning its menu
    pub fn into_menu(self) -> Menu<T> {
        self.menu
    }

    /// Returns the current index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Sets the index, if the specified `index < menu.entries.len()`
    pub fn set_index(&mut self, index: usize) {
        if index < self.menu.entries.len() {
            self.index = index;
        }
    }

    /// Select the entry above the current selection,
    /// unless the first entry is currently selected.
    pub fn up(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    /// Select the entry below the current selection,
    /// unless the last entry is currently selected.
    pub fn down(&mut self) {
        if self.index < self.menu.entries.len() - 1 {
            self.index += 1;
        }
    }

    /// Returns a copy of the currently selected
    /// entry's value.
    pub fn selected(&self) -> T {
        self.menu.entries[self.index].value
    }

    /// Feed input into the menu instance, possibly
    /// changing the selected entry, cancelling the
    /// menu, attempting to exit the program, or
    /// finalising the selection.
    pub fn tick<I>(&mut self, inputs: I) -> Option<MenuOutput<T>>
    where
        I: IntoIterator<Item = Input>,
    {
        for input in inputs {
            match input {
                ETX => return Some(MenuOutput::Quit),
                ESCAPE => return Some(MenuOutput::Cancel),
                RETURN => {
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
                ETX => return Some(MenuOutput::Quit),
                ESCAPE => return Some(MenuOutput::Cancel),
                RETURN => {
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
                Input::MouseMove(coord) => {
                    if let Some(index) = view.menu_index_from_screen_coord(&self.menu, coord) {
                        self.set_index(index);
                    }
                }
                Input::MousePress { coord, .. } => {
                    if let Some(index) = view.menu_index_from_screen_coord(&self.menu, coord) {
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

pub trait MenuIndexFromScreenCoord {
    fn menu_index_from_screen_coord<'a, T: Copy>(
        &self,
        menu: &'a Menu<T>,
        coord: Coord,
    ) -> Option<usize>;
}

impl MenuIndexFromScreenCoord for () {
    fn menu_index_from_screen_coord<'a, T: Copy>(
        &self,
        menu: &'a Menu<T>,
        coord: Coord,
    ) -> Option<usize> {
        None
    }
}

/// Default view of a `MenuInstance`.
pub struct DefaultMenuInstanceView {
    last_coord: Coord,
}

impl DefaultMenuInstanceView {
    pub fn new() -> Self {
        Self {
            last_coord: Coord::new(0, 0),
        }
    }
}

impl MenuIndexFromScreenCoord for DefaultMenuInstanceView {
    fn menu_index_from_screen_coord<'a, T: Copy>(
        &self,
        menu: &'a Menu<T>,
        coord: Coord,
    ) -> Option<usize> {
        let rel_coord = coord - self.last_coord;
        if rel_coord.x < 0 || rel_coord.y < 0 || rel_coord.x >= menu.size.x() as i32
            || rel_coord.y >= menu.entries.len() as i32
        {
            None
        } else {
            Some(rel_coord.y as usize)
        }
    }
}

impl<T: Copy> View<MenuInstance<T>> for DefaultMenuInstanceView {
    fn view<G: ViewGrid>(
        &mut self,
        value: &MenuInstance<T>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.last_coord = offset;
        for (i, entry) in value.menu.entries.iter().enumerate() {
            if i == value.menu.size.y() as usize {
                break;
            }
            let info = if i == value.index {
                &value.menu.selected_info
            } else {
                &value.menu.normal_info
            };
            for (j, ch) in entry.name.chars().enumerate() {
                if j == value.menu.size.x() as usize {
                    break;
                }
                let coord = offset + Coord::new(j as i32, i as i32);
                if let Some(cell) = grid.get_mut(coord, depth) {
                    cell.set_character(ch);
                    info.write_cell(cell);
                }
            }
        }
    }
}

impl<T: Copy> ViewSize<MenuInstance<T>> for DefaultMenuInstanceView {
    fn size(&mut self, data: &MenuInstance<T>) -> Size {
        data.menu.size
    }
}
