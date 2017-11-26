use std::mem;
use cgmath::Vector2;
use context::*;
use grid::*;
use elements::MenuPlace;


#[derive(Debug, Clone)]
struct MonoInner {
    child: ElementHandle,
}
impl MonoInner {
    fn new<E: Into<ElementHandle>>(child: E) -> Self {
        Self {
            child: child.into(),
        }
    }
    fn replace<E: Into<ElementHandle>>(&mut self, new_child: E) -> ElementHandle {
        mem::replace(&mut self.child, new_child.into())
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        self.child.render(grid, seq, offset, depth);
    }
}

#[derive(Debug, Clone)]
pub struct Mono(ElementCell<MonoInner>);

impl Mono {
    pub fn new<E: Into<ElementHandle>>(child: E) -> Self {
        Mono(element_cell(MonoInner::new(child)))
    }
    pub fn replace<E: Into<ElementHandle>>(&self, new_child: E) -> ElementHandle {
        self.0.borrow_mut().replace(new_child)
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
    pub(crate) fn find_menu_place(&self, name: &str) -> Option<MenuPlace> {
        (*self.0).borrow().child.find_menu_place(name)
    }
    pub fn size(&self) -> Vector2<u16> {
        (*self.0).borrow().child.size()
    }
}
