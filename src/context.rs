use std::cell::RefCell;
use std::borrow::Borrow;
use std::rc::Rc;
use std::collections::BTreeMap;
use std::mem;
use cgmath::Vector2;
use terminal::Terminal;
use error::Result;

pub struct Context {
    terminal: Terminal,
}

impl Context {
    pub fn new() -> Result<Self> {
        Terminal::new().map(Self::from_terminal)
    }

    pub fn from_terminal(terminal: Terminal) -> Self {
        Self {
            terminal,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementHandle {
    AbsDiv(AbsDivHandle),
    Text(TextHandle),
}

impl ElementHandle {
    pub fn abs_div(&self) -> Option<&AbsDivHandle> {
        if let &ElementHandle::AbsDiv(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn text(&self) -> Option<&TextHandle> {
        if let &ElementHandle::Text(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbsDiv {
    size: Vector2<u16>,
    children: BTreeMap<String, ElementWithAbsCoord>,
}
#[derive(Debug, Clone)]
struct ElementWithAbsCoord {
    element: ElementHandle,
    coord: Vector2<i16>,
    z_index: i16,
}
impl From<ElementWithAbsCoord> for (ElementHandle, Vector2<i16>, i16) {
    fn from(e: ElementWithAbsCoord) -> Self {
        (e.element, e.coord, e.z_index)
    }
}
impl AbsDiv {
    pub fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        Self {
            size: size.into(),
            children: BTreeMap::new(),
        }
    }
    pub fn set_size<D: Into<Vector2<u16>>>(&mut self, size: D) {
        self.size = size.into();
    }
    pub fn insert<K, E, C>(&mut self, key: K, element: E, coord: C, z_index: i16) -> Option<(ElementHandle, Vector2<i16>, i16)>
        where K: Into<String>,
              E: Into<ElementHandle>,
              C: Into<Vector2<i16>>,
    {
        self.children.insert(key.into(), ElementWithAbsCoord {
            element: element.into(),
            coord: coord.into(),
            z_index,
        }).map(Into::into)
    }
    pub fn remove<K>(&mut self, key: &K) -> Option<(ElementHandle, Vector2<i16>, i16)>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.children.remove(key).map(Into::into)
    }
    pub fn update_coord<K, C>(&mut self, key: &K, coord: C) -> Option<Vector2<i16>>
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
    pub fn update_z_index<K>(&mut self, key: &K, z_index: i16) -> Option<i16>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        if let Some(child) = self.children.get_mut(key) {
            Some(mem::replace(&mut child.z_index, z_index))
        } else {
            None
        }
    }
    pub fn get<K>(&self, key: &K) -> Option<&ElementHandle>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.children.get(key).map(|e| &e.element)
    }
    pub fn into_handle(self) -> AbsDivHandle { AbsDivHandle(Rc::new(RefCell::new(self))) }
}

#[derive(Debug, Clone)]
pub struct AbsDivHandle(Rc<RefCell<AbsDiv>>);
impl AbsDivHandle {
    pub fn set_size<D: Into<Vector2<u16>>>(&self, size: D) {
        self.0.borrow_mut().set_size(size);
    }
    pub fn insert<K, E>(&self, key: K, element: E, coord: Vector2<i16>, z_index: i16) -> Option<(ElementHandle, Vector2<i16>, i16)>
        where K: Into<String>,
              E: Into<ElementHandle>,
    {
        self.0.borrow_mut().insert(key, element, coord, z_index)
    }
    pub fn remove<K>(&mut self, key: &K) -> Option<(ElementHandle, Vector2<i16>, i16)>
        where String: Borrow<K>,
              K: Ord,
    {
        self.0.borrow_mut().remove(key)
    }
    pub fn update_coord<K, C>(&mut self, key: &K, coord: C) -> Option<Vector2<i16>>
        where String: Borrow<K>,
              K: Ord + ?Sized,
              C: Into<Vector2<i16>>,
    {
        self.0.borrow_mut().update_coord(key, coord)
    }
    pub fn update_z_index<K>(&mut self, key: &K, z_index: i16) -> Option<i16>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        self.0.borrow_mut().update_z_index(key, z_index)
    }
    pub fn get<K>(&self, key: &K) -> Option<ElementHandle>
        where String: Borrow<K>,
              K: Ord + ?Sized,
    {
        (*self.0).borrow().get(key).cloned()
    }
}

impl From<AbsDivHandle> for ElementHandle {
    fn from(e: AbsDivHandle) -> Self {
        ElementHandle::AbsDiv(e)
    }
}
impl From<AbsDiv> for AbsDivHandle {
    fn from(e: AbsDiv) -> Self {
        e.into_handle()
    }
}
impl From<AbsDiv> for ElementHandle {
    fn from(e: AbsDiv) -> Self {
        ElementHandle::AbsDiv(e.into())
    }
}


#[derive(Debug, Clone)]
pub struct Text {
    size: Vector2<u16>,
    string: String,
}

impl Text {
    pub fn new<D: Into<Vector2<u16>>, S: Into<String>>(string: S, size: D) -> Self {
        Self {
            size: size.into(),
            string: string.into(),
        }
    }
    pub fn set_size<D: Into<Vector2<u16>>>(&mut self, size: D) {
        self.size = size.into();
    }
    pub fn set<S: Into<String>>(&mut self, new: S) -> String {
        mem::replace(&mut self.string, new.into())
    }
    pub fn get(&self) -> &String {
        &self.string
    }
    pub fn into_handle(self) -> TextHandle { TextHandle(Rc::new(RefCell::new(self))) }
}

#[derive(Debug, Clone)]
pub struct TextHandle(Rc<RefCell<Text>>);

impl TextHandle {
    pub fn set_size<D: Into<Vector2<u16>>>(&self, size: D) {
        self.0.borrow_mut().set_size(size);
    }
    pub fn set<S: Into<String>>(&self, new: S) -> String {
        self.0.borrow_mut().set(new)
    }
    pub fn get(&self) -> String {
        (*self.0).borrow().get().clone()
    }
}

impl From<TextHandle> for ElementHandle {
    fn from(e: TextHandle) -> Self {
        ElementHandle::Text(e)
    }
}
impl From<Text> for ElementHandle {
    fn from(e: Text) -> Self {
        ElementHandle::Text(e.into())
    }
}
impl From<Text> for TextHandle {
    fn from(e: Text) -> Self {
        e.into_handle()
    }
}
