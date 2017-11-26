use ansi_colour::{Colour, colours};
use cgmath::Vector2;
use context::*;
use grid::*;

const FOREGROUND: Colour = colours::WHITE;
const BACKGROUND: Colour = colours::BLACK;

#[derive(Debug, Clone)]
struct Menu {
    strings: Vec<String>,
    index: usize,
}

#[derive(Debug, Clone)]
struct MenuPlaceInner {
    size: Vector2<u16>,
    name: String,
    menu: Option<Menu>,
}

impl MenuPlaceInner {
    fn new<D: Into<Vector2<u16>>, S: Into<String>>(size: D, name: S) -> Self {
        Self {
            size: size.into(),
            name: name.into(),
            menu: None,
        }
    }
    fn set_menu(&mut self, strings: Vec<String>, index: usize) {
        self.menu = Some(Menu { strings, index });
    }
    fn clear_menu(&mut self) {
        self.menu = None;
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        self.menu.as_ref().map(|menu| {
            for (row, string) in menu.strings.iter().enumerate() {
                if row == menu.index {
                    for (col, ch) in string.chars().enumerate() {
                        grid.get_mut(offset + Vector2::new(col, row).cast()).map(|c| {
                            c.update_with_style(seq, ch, depth, BACKGROUND, FOREGROUND, true, true);
                        });
                    }
                } else {
                    for (col, ch) in string.chars().enumerate() {
                        grid.get_mut(offset + Vector2::new(col, row).cast()).map(|c| {
                            c.update_with_style(seq, ch, depth, FOREGROUND, BACKGROUND, false, false);
                        });
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct MenuPlace(ElementCell<MenuPlaceInner>);

impl MenuPlace {
    pub fn new<D: Into<Vector2<u16>>, S: Into<String>>(size: D, name: S) -> Self {
        MenuPlace(element_cell(MenuPlaceInner::new(size, name)))
    }
    pub fn size(&self) -> Vector2<u16> {
        (*self.0).borrow().size
    }
    pub(crate) fn name_matches(&self, name: &str) -> bool {
        (*self.0).borrow().name == name
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
    pub(crate) fn set_menu(&self, strings: Vec<String>, index: usize) {
        self.0.borrow_mut().set_menu(strings, index);
    }
    pub(crate) fn clear_menu(&self) {
        self.0.borrow_mut().clear_menu();
    }
}
