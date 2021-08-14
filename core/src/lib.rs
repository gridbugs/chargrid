pub use chargrid_input as input;
use grid_2d::Grid;
pub use grid_2d::{Coord, Size};
use input::Input;
pub use rgb_int;
pub use rgb_int::Rgba32;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    top_left: Coord,
    bottom_right: Coord,
}

impl BoundingBox {
    pub fn default_with_size(size: Size) -> Self {
        Self {
            top_left: Coord::new(0, 0),
            bottom_right: size.to_coord().unwrap(),
        }
    }

    pub fn size(&self) -> Size {
        (self.bottom_right - self.top_left).to_size().unwrap()
    }

    pub fn coord_relative_to_absolute(&self, coord: Coord) -> Option<Coord> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let absolute_coord = self.top_left + coord;
        if absolute_coord.x < self.bottom_right.x && absolute_coord.y < self.bottom_right.y {
            Some(absolute_coord)
        } else {
            None
        }
    }

    pub fn coord_absolute_to_relative(&self, coord: Coord) -> Option<Coord> {
        if coord.x < self.top_left.x
            || coord.y < self.top_left.y
            || coord.x >= self.bottom_right.x
            || coord.y >= self.bottom_right.y
        {
            return None;
        }
        Some(coord - self.top_left)
    }

    pub fn add_offset(self, offset: Coord) -> Self {
        let top_left = Coord {
            x: (self.top_left.x + offset.x).min(self.bottom_right.x),
            y: (self.top_left.y + offset.y).min(self.bottom_right.y),
        };
        Self { top_left, ..self }
    }

    pub fn constrain_size_by(self, by: Coord) -> Self {
        let bottom_right = Coord {
            x: (self.bottom_right.x - by.x).max(self.top_left.x),
            y: (self.bottom_right.y - by.y).max(self.top_left.y),
        };
        Self {
            bottom_right,
            ..self
        }
    }

    pub fn set_size(self, size: Size) -> Self {
        Self {
            bottom_right: self.top_left + size.to_coord().unwrap(),
            ..self
        }
    }
}

#[derive(Clone, Copy)]
pub struct FrameBufferCell {
    pub character: char,
    pub bold: bool,
    pub underline: bool,
    pub foreground: Rgba32,
    pub background: Rgba32,
    foreground_depth: i8,
    background_depth: i8,
}

pub type FrameBufferIter<'a> = grid_2d::GridIter<'a, FrameBufferCell>;
pub type FrameBufferEnumerate<'a> = grid_2d::GridEnumerate<'a, FrameBufferCell>;
pub type FrameBufferRows<'a> = grid_2d::GridRows<'a, FrameBufferCell>;

impl FrameBufferCell {
    const BLANK: Self = Self {
        character: ' ',
        bold: false,
        underline: false,
        foreground: Rgba32::new_rgb(255, 255, 255),
        background: Rgba32::new_rgb(0, 0, 0),
        foreground_depth: i8::MIN,
        background_depth: i8::MIN,
    };
    fn set_character(&mut self, character: char, depth: i8) {
        if depth >= self.foreground_depth {
            self.character = character;
            self.foreground_depth = depth;
        }
    }
    fn set_bold(&mut self, bold: bool, depth: i8) {
        if depth >= self.foreground_depth {
            self.bold = bold;
            self.foreground_depth = depth;
        }
    }
    fn set_underline(&mut self, underline: bool, depth: i8) {
        if depth >= self.foreground_depth {
            self.underline = underline;
            self.foreground_depth = depth;
        }
    }
    fn set_foreground(&mut self, foreground: Rgba32, depth: i8) {
        if depth >= self.foreground_depth {
            self.foreground = foreground;
            self.foreground_depth = depth;
        }
    }
    fn set_background(&mut self, background: Rgba32, depth: i8) {
        if depth >= self.background_depth {
            self.background = background;
            self.background_depth = depth;
        }
    }
}

pub struct FrameBuffer {
    grid: Grid<FrameBufferCell>,
}

impl FrameBuffer {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_copy(size, FrameBufferCell::BLANK),
        }
    }

    pub fn size(&self) -> Size {
        self.grid.size()
    }

    pub fn resize(&mut self, size: Size) {
        self.grid = Grid::new_copy(size, FrameBufferCell::BLANK);
    }

    pub fn clear_with_background(&mut self, background: Rgba32) {
        for cell in self.grid.iter_mut() {
            *cell = FrameBufferCell {
                background,
                ..FrameBufferCell::BLANK
            };
        }
    }

    pub fn clear(&mut self) {
        for cell in self.grid.iter_mut() {
            *cell = FrameBufferCell::BLANK;
        }
    }

    pub fn enumerate(&self) -> FrameBufferEnumerate {
        self.grid.enumerate()
    }

    pub fn iter(&self) -> FrameBufferIter {
        self.grid.iter()
    }

    pub fn rows(&self) -> FrameBufferRows {
        self.grid.rows()
    }

    pub fn set_cell(&mut self, coord: Coord, depth: i8, render_cell: RenderCell) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if cell.foreground_depth <= depth || cell.background_depth <= depth {
                if let Some(character) = render_cell.character {
                    cell.set_character(character, depth);
                }
                if let Some(bold) = render_cell.style.bold {
                    cell.set_bold(bold, depth);
                }
                if let Some(underline) = render_cell.style.underline {
                    cell.set_underline(underline, depth);
                }
                if let Some(foreground) = render_cell.style.foreground {
                    let foreground_blended = foreground.alpha_composite(cell.background);
                    cell.set_foreground(foreground_blended, depth);
                }
                if let Some(background) = render_cell.style.background {
                    let background_blended = background.alpha_composite(cell.background);
                    cell.set_background(background_blended, depth);
                }
            }
        }
    }

    pub fn default_ctx<'a>(&self) -> Ctx<'a> {
        Ctx::default_with_bounding_box_size(self.size())
    }

    pub fn set_cell_relative_to_ctx<'a>(
        &mut self,
        ctx: Ctx<'a>,
        coord: Coord,
        depth: i8,
        render_cell: RenderCell,
    ) {
        if let Some(absolute_coord) = ctx.bounding_box.coord_relative_to_absolute(coord) {
            let absolute_depth = depth + ctx.depth;
            self.set_cell(
                absolute_coord,
                absolute_depth,
                render_cell.apply_tint(ctx.tint),
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub foreground: Option<Rgba32>,
    pub background: Option<Rgba32>,
}

impl Style {
    pub const DEFAULT: Self = Self {
        bold: None,
        underline: None,
        foreground: None,
        background: None,
    };

    fn apply_tint(self, tint: &dyn Tint) -> Self {
        Self {
            foreground: self.foreground.map(|r| tint.tint(r)),
            background: self.background.map(|r| tint.tint(r)),
            ..self
        }
    }

    pub const fn new() -> Self {
        Self::DEFAULT
    }

    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            bold: Some(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            underline: Some(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgba32) -> Self {
        Self {
            foreground: Some(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgba32) -> Self {
        Self {
            background: Some(background),
            ..self
        }
    }
    pub const fn without_bold(self) -> Self {
        Self { bold: None, ..self }
    }
    pub const fn without_underline(self) -> Self {
        Self {
            underline: None,
            ..self
        }
    }
    pub const fn without_foreground(self) -> Self {
        Self {
            foreground: None,
            ..self
        }
    }
    pub const fn without_background(self) -> Self {
        Self {
            background: None,
            ..self
        }
    }
    pub fn coalesce(self, other: Self) -> Self {
        Self {
            bold: (self.bold.or(other.bold)),
            underline: (self.underline.or(other.underline)),
            foreground: (self.foreground.or(other.foreground)),
            background: (self.background.or(other.background)),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderCell {
    pub character: Option<char>,
    pub style: Style,
}

impl RenderCell {
    pub const BLANK: Self = Self {
        character: None,
        style: Style::DEFAULT,
    };

    fn apply_tint(self, tint: &dyn Tint) -> Self {
        Self {
            style: self.style.apply_tint(tint),
            ..self
        }
    }

    pub const fn character(&self) -> Option<char> {
        self.character
    }
    pub const fn bold(&self) -> Option<bool> {
        self.style.bold
    }
    pub const fn underline(&self) -> Option<bool> {
        self.style.underline
    }
    pub const fn foreground(&self) -> Option<Rgba32> {
        self.style.foreground
    }
    pub const fn background(&self) -> Option<Rgba32> {
        self.style.background
    }
    pub const fn with_character(self, character: char) -> Self {
        Self {
            character: Some(character),
            ..self
        }
    }
    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            style: self.style.with_bold(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            style: self.style.with_underline(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgba32) -> Self {
        Self {
            style: self.style.with_foreground(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgba32) -> Self {
        Self {
            style: self.style.with_background(background),
            ..self
        }
    }
    pub const fn without_character(self) -> Self {
        Self {
            character: None,
            ..self
        }
    }
    pub const fn without_bold(self) -> Self {
        Self {
            style: self.style.without_bold(),
            ..self
        }
    }
    pub const fn without_underline(self) -> Self {
        Self {
            style: self.style.without_underline(),
            ..self
        }
    }
    pub const fn without_foreground(self) -> Self {
        Self {
            style: self.style.without_foreground(),
            ..self
        }
    }
    pub const fn without_background(self) -> Self {
        Self {
            style: self.style.without_background(),
            ..self
        }
    }
    pub const fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }
}

impl Default for RenderCell {
    fn default() -> Self {
        Self::BLANK
    }
}

pub trait Tint {
    fn tint(&self, rgba32: Rgba32) -> Rgba32;
}

pub struct TintIdentity;
impl Tint for TintIdentity {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        rgba32
    }
}

impl<F: Fn(Rgba32) -> Rgba32> Tint for F {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        (&self)(rgba32)
    }
}

pub struct TintDim(pub u8);
impl Tint for TintDim {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        rgba32.normalised_scalar_mul(self.0)
    }
}

#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub tint: &'a dyn Tint,
    pub depth: i8,
    pub bounding_box: BoundingBox,
}

impl<'a> Ctx<'a> {
    pub fn default_with_bounding_box_size(size: Size) -> Self {
        Self {
            tint: &TintIdentity,
            depth: 0,
            bounding_box: BoundingBox::default_with_size(size),
        }
    }

    pub fn add_offset(self, offset: Coord) -> Self {
        Self {
            bounding_box: self.bounding_box.add_offset(offset),
            ..self
        }
    }

    pub fn add_depth(self, depth_delta: i8) -> Self {
        Self {
            depth: self.depth + depth_delta,
            ..self
        }
    }

    pub fn constrain_size_by(self, by: Coord) -> Self {
        Self {
            bounding_box: self.bounding_box.constrain_size_by(by),
            ..self
        }
    }

    pub fn set_size(self, size: Size) -> Self {
        Self {
            bounding_box: self.bounding_box.set_size(size),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Input(Input),
    Tick(Duration),
    Peek,
}

impl Event {
    pub fn input(self) -> Option<Input> {
        if let Self::Input(input) = self {
            Some(input)
        } else {
            None
        }
    }

    pub fn tick(self) -> Option<Duration> {
        if let Self::Tick(duration) = self {
            Some(duration)
        } else {
            None
        }
    }

    pub fn is_peek(self) -> bool {
        if let Self::Peek = self {
            true
        } else {
            false
        }
    }
}

pub trait Component {
    type Output;
    type State: ?Sized;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer);
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output;
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size;
}

pub mod app {
    #[derive(Clone, Copy, Debug)]
    pub struct Exit;
    pub type Output = Option<Exit>;
}

/// types/traits/modules useful for implementing `Component` and friends
pub mod prelude {
    pub use super::{
        app, input, Component, Coord, Ctx, Event, FrameBuffer, RenderCell, Rgba32, Size, Style,
    };
    pub use std::time::Duration;
}
