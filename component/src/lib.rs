pub use chargrid_input as input;
use grid_2d::Grid;
pub use grid_2d::{Coord, Size};
use input::Input;
pub use rgba32::Rgba32;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub coord: Coord,
    pub size: Size,
}

impl BoundingBox {
    pub fn default_with_size(size: Size) -> Self {
        Self {
            coord: Coord::new(0, 0),
            size,
        }
    }

    pub fn coord_relative_to_absolute(&self, coord: Coord) -> Option<Coord> {
        let absolute_coord = self.coord + coord;
        if coord.x >= self.coord.x && coord.y >= self.coord.y && self.size.is_valid(coord) {
            Some(absolute_coord)
        } else {
            None
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
        foreground_depth: 0,
        background_depth: 0,
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
                    cell.set_foreground(foreground, depth);
                }
                if let Some(background) = render_cell.style.background {
                    cell.set_background(background, depth);
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
            self.set_cell(absolute_coord, absolute_depth, render_cell);
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
}

pub enum Event {
    Input(Input),
    Tick(Duration),
    Peek,
}

pub trait Component<S> {
    type Output;
    fn render(&self, state: &S, ctx: Ctx, fb: &mut FrameBuffer);
    fn update(&mut self, state: &mut S, ctx: Ctx, event: Event) -> Self::Output;
}

pub trait PureComponent {
    type PureOutput;
    fn pure_render(&self, ctx: Ctx, fb: &mut FrameBuffer);
    fn pure_update(&mut self, ctx: Ctx, event: Event) -> Self::PureOutput;
}

impl<T: PureComponent, S> Component<S> for T {
    type Output = T::PureOutput;
    fn render(&self, _: &S, ctx: Ctx, fb: &mut FrameBuffer) {
        self.pure_render(ctx, fb);
    }
    fn update(&mut self, _: &mut S, ctx: Ctx, event: Event) -> Self::Output {
        self.pure_update(ctx, event)
    }
}

pub trait FunctionalComponent<S> {
    type FunctionalOutput;
    fn functional_update(
        &mut self,
        state: &mut S,
        ctx: Ctx,
        event: Event,
    ) -> Option<Self::FunctionalOutput>;
}

pub enum ControlFlow {
    Exit,
}

/// Temporary wrapper to convert a Component into an App
pub struct AppWrapper<C> {
    pub component: C,
    pub frame_buffer: FrameBuffer,
}

impl<C> chargrid_app::App for AppWrapper<C>
where
    C: Component<(), Output = Option<ControlFlow>>,
{
    fn on_input(&mut self, input: Input) -> Option<chargrid_app::ControlFlow> {
        self.component
            .update(
                &mut (),
                self.frame_buffer.default_ctx(),
                Event::Input(input),
            )
            .map(|cf| match cf {
                ControlFlow::Exit => chargrid_app::ControlFlow::Exit,
            })
    }
    fn on_frame<F, CM>(
        &mut self,
        since_last_frame: Duration,
        _view_context: chargrid_app::ViewContext<CM>,
        frame: &mut F,
    ) -> Option<chargrid_app::ControlFlow>
    where
        F: chargrid_app::Frame,
        CM: chargrid_app::ColModify,
    {
        self.frame_buffer.clear();
        self.component
            .render(&(), self.frame_buffer.default_ctx(), &mut self.frame_buffer);
        for (coord, cell) in self.frame_buffer.enumerate() {
            frame.set_cell_absolute(
                coord,
                1,
                chargrid_render::ViewCell {
                    character: Some(cell.character),
                    style: chargrid_render::Style {
                        bold: Some(cell.bold),
                        underline: Some(cell.underline),
                        foreground: Some(chargrid_render::Rgb24::new(
                            cell.foreground.r,
                            cell.foreground.g,
                            cell.foreground.b,
                        )),
                        background: Some(chargrid_render::Rgb24::new(
                            cell.background.r,
                            cell.background.g,
                            cell.background.b,
                        )),
                    },
                },
            );
        }
        self.component
            .update(
                &mut (),
                self.frame_buffer.default_ctx(),
                Event::Tick(since_last_frame),
            )
            .map(|cf| match cf {
                ControlFlow::Exit => chargrid_app::ControlFlow::Exit,
            })
    }
}
