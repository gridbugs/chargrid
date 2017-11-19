use std::cell::RefCell;
use std::borrow::Borrow;
use std::rc::Rc;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::mem;
use cgmath::Vector2;
use terminal::Terminal;
use input::Input;
use error::Result;

#[derive(Debug, Clone)]
struct Cell {
    seq: u64,
    byte: u8,
    z_index: i16,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            seq: 0,
            byte: b' ',
            z_index: 0,
        }
    }
}

impl Cell {
    fn update(&mut self, seq: u64, byte: u8, z_index: i16) {
        if seq > self.seq || (seq == self.seq && z_index >= self.z_index) {
            self.seq = seq;
            self.byte = byte;
            self.z_index = z_index;
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    size: Vector2<u16>,
    cells: Vec<Cell>,
}

impl Grid {
    fn new(size: Vector2<u16>) -> Self {

        let num_cells = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(num_cells);
        cells.resize(num_cells, Default::default());

        Self {
            size,
            cells,
        }
    }

    fn resize(&mut self, size: Vector2<u16>) {
        let num_cells = (size.x * size.y) as usize;
        self.cells.resize(num_cells, Default::default());
        self.size = size;
    }

    fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut Cell> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }

    fn render(&mut self, seq: u64, offset: Vector2<i16>, z_index: i16, element: &ElementHandle) {
        match element {
            &ElementHandle::AbsDiv(ref div) => self.render_abs_div(seq, offset, z_index, (*div.0).borrow().deref()),
            &ElementHandle::Text(ref text) => self.render_text(seq, offset, z_index, (*text.0).borrow().deref()),
        }
    }

    fn render_abs_div(&mut self, seq: u64, offset: Vector2<i16>, z_index: i16, abs_div: &AbsDiv) {
        for child in abs_div.children.values() {
            self.render(seq, offset + child.coord, child.z_index.unwrap_or(z_index), &child.element);
        }
    }

    fn render_text(&mut self, seq: u64, offset: Vector2<i16>, z_index: i16, text: &Text) {
        let bottom_right_abs = offset + text.size.cast();
        let mut coord = offset;
        for byte in text.string.as_str().as_bytes().iter().cloned() {
            match byte {
                b'\n' => {
                    coord.x = offset.x;
                    coord.y += 1;
                    if coord.y == bottom_right_abs.y {
                        break;
                    }
                }
                b'\r' => {
                    coord.x = offset.x;
                }
                _ => {
                    if let Some(cell) = self.get_mut(coord) {
                        cell.update(seq, byte, z_index);
                    }
                    coord.x += 1;
                    if coord.x == bottom_right_abs.x {
                        coord.x = offset.x;
                        coord.y += 1;
                        if coord.y == bottom_right_abs.y {
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub struct Context {
    terminal: Terminal,
    seq: u64,
    grid: Grid,
}

impl Context {
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    pub fn from_terminal(terminal: Terminal) -> Result<Self> {

        let grid = Grid::new(terminal.size()?);

        Ok(Self {
            terminal,
            grid,
            seq: 0,
        })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.size()?;
        if size != self.grid.size {
            self.grid.resize(size);
        }

        Ok(())
    }

    pub fn render(&mut self, root: &ElementHandle) -> Result<()> {
        self.resize_if_necessary()?;
        self.seq += 1;

        self.grid.render(self.seq, Vector2::new(0, 0), 0, &root);
        self.send_grid_contents();
        self.terminal.flush_buffer()?;

        Ok(())
    }

    fn send_grid_contents(&mut self) {
        for cell in self.grid.cells.iter() {
            self.terminal.add_byte_to_buffer(cell.byte);
        }
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        self.terminal.wait_input()
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
    z_index: Option<i16>,
}
impl From<ElementWithAbsCoord> for (ElementHandle, Vector2<i16>, Option<i16>) {
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
    pub fn insert<K, E, C>(&mut self, key: K, element: E, coord: C, z_index: Option<i16>)
        -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
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
    pub fn remove<K>(&mut self, key: &K) -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
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
    pub fn update_z_index<K>(&mut self, key: &K, z_index: Option<i16>) -> Option<Option<i16>>
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
    pub fn insert<K, E, C>(&self, key: K, element: E, coord: C, z_index: Option<i16>)
        -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
        where K: Into<String>,
              E: Into<ElementHandle>,
              C: Into<Vector2<i16>>,
    {
        self.0.borrow_mut().insert(key, element, coord, z_index)
    }
    pub fn remove<K>(&mut self, key: &K) -> Option<(ElementHandle, Vector2<i16>, Option<i16>)>
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
    pub fn update_z_index<K>(&mut self, key: &K, z_index: Option<i16>) -> Option<Option<i16>>
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
