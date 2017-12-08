use std::cmp;
use cgmath::Vector2;
use prototty::*;
use common::TextInfo;
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
    pub size: Vector2<u16>,
    pub selected_info: TextInfo,
    pub normal_info: TextInfo,
}

fn selected_info() -> TextInfo {
    TextInfo::default()
        .bold()
        .foreground_colour(DEFAULT_BG)
        .backrgound_colour(DEFAULT_FG)
}

impl<T: Copy> Menu<T> {
    /// Create a new menu.
    pub fn new<S, V>(mut e: Vec<(S, T)>, size: V) -> Self
        where S: Into<String>,
              V: Into<Vector2<u16>>,
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
        where S: Into<String>,
    {
        let entries: Vec<MenuEntry<T>> = e.drain(..).map(Into::into).collect();
        let width = entries.iter().fold(0, |acc, e| cmp::max(acc, e.name.len()));
        let height = entries.len();
        Self {
            entries,
            size: Vector2::new(width as u16, height as u16),
            normal_info: Default::default(),
            selected_info: selected_info(),
        }
    }
}

impl<T: Copy> View for Menu<T> {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        for (i, entry) in self.entries.iter().enumerate() {
            if i == self.size.y as usize {
                break;
            }
            for (j, ch) in entry.name.chars().enumerate() {
                if j == self.size.x as usize {
                    break;
                }
                let coord = offset + Vector2::new(j, i).cast();
                if let Some(cell) = grid.get_mut(coord) {
                    cell.update_with_style(ch, depth,
                                           self.normal_info.foreground_colour,
                                           self.normal_info.backrgound_colour,
                                           self.normal_info.bold,
                                           self.normal_info.underline);
                }
            }
        }
    }
}

/// An instance of a menu, with a selected entry.
/// A `MenuInstance` can be run using a `MenuRunner`,
/// and a selection finalised. When a `MenuInstance`
/// is rendered, the currently-selected entry is
/// rendered using the `Menu`'s `selected_info`.
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
    /// Returns `None` if `Menu` has fewer than `index - 1` entries.
    pub fn with_index(menu: Menu<T>, index: usize) -> Option<Self> {
        if index < menu.entries.len() {
            Some(Self {
                menu,
                index,
            })
        } else {
            None
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
}

impl<T: Copy> View for MenuInstance<T> {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        for (i, entry) in self.menu.entries.iter().enumerate() {
            if i == self.menu.size.y as usize {
                break;
            }
            let info = if i == self.index {
                &self.menu.selected_info
            } else {
                &self.menu.normal_info
            };
            for (j, ch) in entry.name.chars().enumerate() {
                if j == self.menu.size.x as usize {
                    break;
                }
                let coord = offset + Vector2::new(j, i).cast();
                if let Some(cell) = grid.get_mut(coord) {
                    cell.update_with_style(ch, depth,
                                           info.foreground_colour,
                                           info.backrgound_colour,
                                           info.bold,
                                           info.underline);
                }
            }
        }
    }
}

impl<T: Copy> ViewSize for MenuInstance<T> {
    fn size(&self) -> Vector2<u16> {
        self.menu.size
    }
}
