use std::slice;
use std::mem;
use prototty::*;
use cgmath::Vector2;
use ansi_colour::Colour;
use defaults::*;
use super::iterators::*;

const BLANK: char = ' ';

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
            character: BLANK,
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            bold: false,
            underline: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasBuffer {
    size: Vector2<u16>,
    cells: Vec<CanvasCell>,
}

impl CanvasBuffer {
    pub fn new(size: Vector2<u16>) -> Self {
        let capacity = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Default::default());
        Self {
            size,
            cells,
        }
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CanvasError {
    DifferentBufferSizes {
        current: Vector2<u16>,
        new: Vector2<u16>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    size: Vector2<u16>,
    cells: Vec<CanvasCell>,
}

impl Canvas {
    pub fn new<D: Into<Vector2<u16>>>(size: D) -> Self {
        let size = size.into();
        let capacity = (size.x * size.y) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Default::default());
        Self {
            size,
            cells,
        }
    }
    pub fn set_size<D: Into<Vector2<u16>>>(&mut self, size: D) {
        let size = size.into();
        let capacity = (size.x * size.y) as usize;
        self.size = size;
        self.cells = Vec::with_capacity(capacity);
    }
    pub fn make_buffer(&self) -> CanvasBuffer {
        CanvasBuffer::new(self.size)
    }
    pub fn swap_buffer(&mut self, buffer: &mut CanvasBuffer)
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
}

impl View for Canvas {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        for (coord, canvas_cell) in izip!(CoordIter::new(self.size), self.cells.iter()) {
            let coord = offset + coord;
            if let Some(terminal_cell) = grid.get_mut(coord) {
                terminal_cell.update_with_style(canvas_cell.character, depth,
                                                canvas_cell.foreground_colour,
                                                canvas_cell.background_colour,
                                                canvas_cell.bold,
                                                canvas_cell.underline);
            }
        }
    }
}

impl ViewSize for Canvas {
    fn size(&self) -> Vector2<u16> { self.size }
}
