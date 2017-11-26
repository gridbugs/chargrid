use std::mem;
use std::slice;
use ansi_colour::Colour;
use cgmath::Vector2;
use context::*;
use grid::*;
use iterators::*;
use defaults::*;

#[derive(Debug, Clone, Copy)]
pub struct CanvasCell {
    pub character: char,
    pub foreground_colour: Colour,
    pub background_colour: Colour,
    pub bold: bool,
    pub underline: bool,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            character: DEFAULT_CH,
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasBuffer {
    size: Vector2<u16>,
    cells: Vec<CanvasCell>,
}

impl CanvasBuffer {
    fn new(size: Vector2<u16>) -> Self {
        let capacity = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Default::default());
        Self {
            size,
            cells,
        }
    }
    pub fn size(&self) -> Vector2<u16> { self.size }
    pub fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut CanvasCell> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let coord: Vector2<u16> = coord.cast();
        if coord.x >= self.size.x || coord.y >= self.size.y {
            return None;
        }
        Some(&mut self.cells[(coord.y * self.size.x + coord.x) as usize])
    }
    pub fn coords(&self) -> CoordIter {
        CoordIter::new(self.size)
    }
    pub fn iter(&self) -> slice::Iter<CanvasCell> {
        self.cells.iter()
    }
    pub fn iter_mut(&mut self) -> slice::IterMut<CanvasCell> {
        self.cells.iter_mut()
    }
    pub fn enumerate(&self) -> CoordEnumerate<CanvasCell> {
        CoordEnumerate::new(self.coords(), self.iter())
    }
    pub fn enumerate_mut(&mut self) -> CoordEnumerateMut<CanvasCell> {
        CoordEnumerateMut::new(self.coords(), self.iter_mut())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CanvasError {
    DifferentBufferSizes {
        current: Vector2<u16>,
        new: Vector2<u16>,
    },
}

#[derive(Debug, Clone)]
struct CanvasInner {
    size: Vector2<u16>,
    cells: Vec<CanvasCell>,
}

impl CanvasInner {
    fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        let size = size.into();
        let capacity = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Default::default());
        Self {
            size,
            cells,
        }
    }
    fn set_size<D: Into<Vector2<u16>>>(&mut self, size: D) {
        self.size = size.into();
    }
    fn make_buffer(&self) -> CanvasBuffer {
        CanvasBuffer::new(self.size)
    }
    fn swap_buffer(&mut self, buffer: &mut CanvasBuffer)
        -> ::std::result::Result<(), CanvasError>
    {
        if self.size != buffer.size {
            return Err(CanvasError::DifferentBufferSizes {
                current: self.size,
                new: buffer.size,
            });
        }

        mem::swap(&mut self.cells, &mut buffer.cells);

        Ok(())
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        for (coord, canvas_cell) in izip!(CoordIter::new(self.size), self.cells.iter()) {
            let coord = offset + coord;
            if let Some(terminal_cell) = grid.get_mut(coord) {
                terminal_cell.update_with_style(seq, canvas_cell.character, depth,
                                                canvas_cell.foreground_colour,
                                                canvas_cell.background_colour,
                                                canvas_cell.bold,
                                                canvas_cell.underline);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Canvas(ElementCell<CanvasInner>);

impl Canvas {
    pub fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        Canvas(element_cell(CanvasInner::new(size)))
    }
    pub fn set_size<D: Into<Vector2<u16>>>(&self, size: D) {
        self.0.borrow_mut().set_size(size);
    }
    pub fn make_buffer(&self) -> CanvasBuffer {
        (*self.0).borrow().make_buffer()
    }
    pub fn swap_buffer(&self, buffer: &mut CanvasBuffer)
        -> ::std::result::Result<(), CanvasError>
    {
        self.0.borrow_mut().swap_buffer(buffer)
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
}
