use std::cmp;
use cgmath::Vector2;
use prototty::*;
use text_info::*;
use defaults::*;

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

pub struct MenuInstance<T: Copy> {
    pub menu: Menu<T>,
    index: usize,
}

impl<T: Copy> MenuInstance<T> {
    pub fn new(menu: Menu<T>) -> Self {
        Self {
            menu,
            index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn up(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    pub fn down(&mut self) {
        if self.index < self.menu.entries.len() - 1 {
            self.index += 1;
        }
    }

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
    fn size(&self) -> Vector2<u16> {
        self.menu.size
    }
}
