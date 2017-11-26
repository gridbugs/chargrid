use std::mem;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use cgmath::Vector2;
use context::*;
use grid::*;

#[derive(Debug, Clone)]
struct AbsDivInner {
    size: Vector2<u16>,
    children: BTreeMap<String, ElementWithAbsCoord>,
}
#[derive(Debug, Clone)]
struct ElementWithAbsCoord {
    element: ElementHandle,
    coord: Vector2<i16>,
    depth: Option<i16>,
}
impl From<ElementWithAbsCoord> for (ElementHandle, Vector2<i16>, Option<i16>) {
    fn from(e: ElementWithAbsCoord) -> Self {
        (e.element, e.coord, e.depth)
    }
}
impl AbsDivInner {
    fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        Self {
            size: size.into(),
            children: BTreeMap::new(),
        }
    }
    fn set_size<D: Into<Vector2<u16>>>(&mut self, size: D) {
        self.size = size.into();
    }
    fn insert<K, E, C>(&mut self, key: K, element: E, coord: C, depth: Option<i16>)
        -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
        where K: Into<String>,
              E: Into<ElementHandle>,
              C: Into<Vector2<i16>>,
    {
        self.children.insert(key.into(), ElementWithAbsCoord {
            element: element.into(),
            coord: coord.into(),
            depth,
        }).map(Into::into)
    }
    fn remove<K>(&mut self, key: &K) -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.children.remove(key).map(Into::into)
    }
    fn update_coord<K, C>(&mut self, key: &K, coord: C) -> Option<Vector2<i16>>
        where String: Borrow<K>,
              K: Ord + ?Sized,
              C: Into<Vector2<i16>>,
    {
        if let Some(child) = self.children.get_mut(key) {
            Some(mem::replace(&mut child.coord, coord.into()))
        } else {
            None
        }
    }
    fn update_depth<K>(&mut self, key: &K, depth: Option<i16>) -> Option<Option<i16>>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        if let Some(child) = self.children.get_mut(key) {
            Some(mem::replace(&mut child.depth, depth))
        } else {
            None
        }
    }
    fn get<K>(&self, key: &K) -> Option<&ElementHandle>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.children.get(key).map(|e| &e.element)
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        for child in self.children.values() {
            child.element.render(grid, seq, offset + child.coord, child.depth.unwrap_or(depth));
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbsDiv(ElementCell<AbsDivInner>);

impl AbsDiv {
    pub fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        AbsDiv(element_cell(AbsDivInner::new(size)))
    }
    pub fn set_size<D: Into<Vector2<u16>>>(&self, size: D) {
        self.0.borrow_mut().set_size(size);
    }
    pub fn insert<K, E, C>(&self, key: K, element: E, coord: C, depth: Option<i16>)
        -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
        where K: Into<String>,
              E: Into<ElementHandle>,
              C: Into<Vector2<i16>>,
    {
        self.0.borrow_mut().insert(key, element, coord, depth)
    }
    pub fn remove<K>(&self, key: &K) -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.0.borrow_mut().remove(key)
    }
    pub fn update_coord<K, C>(&self, key: &K, coord: C) -> Option<Vector2<i16>>
        where String: Borrow<K>,
              K: Ord + ?Sized,
              C: Into<Vector2<i16>>,
    {
        self.0.borrow_mut().update_coord(key, coord)
    }
    pub fn update_depth<K>(&mut self, key: &K, depth: Option<i16>) -> Option<Option<i16>>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.0.borrow_mut().update_depth(key, depth)
    }
    pub fn get<K>(&self, key: &K) -> Option<ElementHandle>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        (*self.0).borrow().get(key).cloned()
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
}
