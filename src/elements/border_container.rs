use std::borrow::Borrow;
use std::mem;
use cgmath::Vector2;
use ansi_colour::Colour;
use context::*;
use grid::*;
use defaults::*;
use elements::MenuPlace;

const HORIZONTAL: char = '─';
const VERTICAL: char = '│';
const TOP_LEFT: char = '┌';
const TOP_RIGHT: char = '┐';
const BOTTOM_LEFT: char = '└';
const BOTTOM_RIGHT: char = '┘';

#[derive(Debug, Clone)]
struct BorderContainerInner {
    child: ElementHandle,
    fg: Colour,
    bg: Colour,
    title: Option<String>,
}

impl BorderContainerInner {
    fn new<E: Into<ElementHandle>>(child: E, fg: Colour, bg: Colour) -> Self {
        Self {
            child: child.into(),
            fg,
            bg,
            title: None,
        }
    }
    fn replace<E: Into<ElementHandle>>(&mut self, new_child: E) -> ElementHandle {
        mem::replace(&mut self.child, new_child.into())
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        self.child.render(grid, seq, offset + Vector2::new(1, 1), depth);
        let span = self.child.size().cast() + Vector2::new(1, 1);

        if let Some(c) = grid.get_mut(offset) {
            c.update_with_colour(seq, TOP_LEFT, depth, self.fg, self.bg);
        }

        if let Some(c) = grid.get_mut(offset + Vector2::new(span.x, 0)) {
            c.update_with_colour(seq, TOP_RIGHT, depth, self.fg, self.bg);
        }

        if let Some(c) = grid.get_mut(offset + Vector2::new(0, span.y)) {
            c.update_with_colour(seq, BOTTOM_LEFT, depth, self.fg, self.bg);
        }

        if let Some(c) = grid.get_mut(offset + span) {
            c.update_with_colour(seq, BOTTOM_RIGHT, depth, self.fg, self.bg);
        }


        let title_offset = if let Some(title) = self.title.as_ref() {
            let before = offset + Vector2::new(1, 0);
            let after = offset + Vector2::new(title.len() as i16 + 2, 0);

            if let Some(c) = grid.get_mut(before) {
                c.update_with_colour(seq, '┤', depth, self.fg, self.bg);
            }
            if let Some(c) = grid.get_mut(after) {
                c.update_with_colour(seq, '├', depth, self.fg, self.bg);
            }

            for (index, ch) in title.chars().enumerate() {
                let coord = offset + Vector2::new(index as i16 + 2, 0);
                if let Some(c) = grid.get_mut(coord) {
                    c.update_with_colour(seq, ch, depth, self.fg, self.bg);
                }
            }

            title.len() as i16 + 2
        } else {
            0
        };

        for i in (1 + title_offset)..span.x {
            if let Some(c) = grid.get_mut(offset + Vector2::new(i, 0)) {
                c.update_with_colour(seq, HORIZONTAL, depth, self.fg, self.bg);
            }
        }
        for i in 1..span.x {
            if let Some(c) = grid.get_mut(offset + Vector2::new(i, span.y)) {
                c.update_with_colour(seq, HORIZONTAL, depth, self.fg, self.bg);
            }
        }

        for i in 1..span.y {
            if let Some(c) = grid.get_mut(offset + Vector2::new(0, i)) {
                c.update_with_colour(seq, VERTICAL, depth, self.fg, self.bg);
            }
            if let Some(c) = grid.get_mut(offset + Vector2::new(span.x, i)) {
                c.update_with_colour(seq, VERTICAL, depth, self.fg, self.bg);
            }
        }
    }
    fn size(&self) -> Vector2<u16> {
        self.child.size() + Vector2::new(2, 2)
    }
}

#[derive(Debug, Clone)]
pub struct BorderContainer(ElementCell<BorderContainerInner>);

impl BorderContainer {
    pub fn new<E: Into<ElementHandle>>(child: E) -> Self {
        BorderContainer(element_cell(BorderContainerInner::new(child, DEFAULT_FG, DEFAULT_BG)))
    }
    pub fn set_foreground(&self, fg: Colour) {
        self.0.borrow_mut().fg = fg;
    }
    pub fn set_background(&self, bg: Colour) {
        self.0.borrow_mut().bg = bg;
    }
    pub fn set_title<S: Borrow<str>>(&self, title: S) {
        self.0.borrow_mut().title = Some(title.borrow().to_string());
    }
    pub fn clear_title(&self) {
        self.0.borrow_mut().title = None;
    }
    pub fn replace<E: Into<ElementHandle>>(&mut self, new_child: E) -> ElementHandle {
        self.0.borrow_mut().replace(new_child)
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
    pub(crate) fn find_menu_place(&self, name: &str) -> Option<MenuPlace> {
        (*self.0).borrow().child.find_menu_place(name)
    }
    pub fn size(&self) -> Vector2<u16> {
        (*self.0).borrow().size()
    }
}
