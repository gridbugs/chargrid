use std::mem;
use cgmath::Vector2;
use context::*;
use grid::*;

const HORIZONTAL: char = '─';
const VERTICAL: char = '│';
const TOP_LEFT: char = '┌';
const TOP_RIGHT: char = '┐';
const BOTTOM_LEFT: char = '└';
const BOTTOM_RIGHT: char = '┘';

#[derive(Debug, Clone)]
struct BorderContainerInner {
    child: ElementHandle,
}

impl BorderContainerInner {
    fn new<E: Into<ElementHandle>>(child: E) -> Self {
        Self {
            child: child.into(),
        }
    }
    fn replace<E: Into<ElementHandle>>(&mut self, new_child: E) -> ElementHandle {
        mem::replace(&mut self.child, new_child.into())
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        self.child.render(grid, seq, offset + Vector2::new(1, 1), depth);
        let span = self.child.size().cast() + Vector2::new(1, 1);
        grid.get_mut(offset).map(|c| c.ch = TOP_LEFT);
        grid.get_mut(offset + Vector2::new(span.x, 0)).map(|c| c.ch = TOP_RIGHT);
        grid.get_mut(offset + Vector2::new(0, span.y)).map(|c| c.ch = BOTTOM_LEFT);
        grid.get_mut(offset + span).map(|c| c.ch = BOTTOM_RIGHT);
        for i in 1..span.x {
            grid.get_mut(offset + Vector2::new(i, 0)).map(|c| c.ch = HORIZONTAL);
            grid.get_mut(offset + Vector2::new(i, span.y)).map(|c| c.ch = HORIZONTAL);
        }
        for i in 1..span.y {
            grid.get_mut(offset + Vector2::new(0, i)).map(|c| c.ch = VERTICAL);
            grid.get_mut(offset + Vector2::new(span.x, i)).map(|c| c.ch = VERTICAL);
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
        BorderContainer(element_cell(BorderContainerInner::new(child)))
    }
    pub fn replace<E: Into<ElementHandle>>(&mut self, new_child: E) -> ElementHandle {
        self.0.borrow_mut().replace(new_child)
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
    pub(crate) fn size(&self) -> Vector2<u16> {
        (*self.0).borrow().size()
    }
}
