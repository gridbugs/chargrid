use gfx;
use gfx_device_gl;
use gfx_glyph;
use gfx_window_glutin;
use glutin;

use gfx::Device;
use gfx_glyph::{GlyphCruncher, HorizontalAlign, Layout, VerticalAlign};
use glutin::dpi::{LogicalSize, PhysicalSize};
use glutin::Event;

use background::*;
use formats::*;
use input::*;
use prototty_grid::*;
use prototty_input::*;
use prototty_render::*;

type Resources = gfx_device_gl::Resources;

const UNDERLINE_WIDTH_RATIO: u32 = 15;
const UNDERLINE_POSITION_RATIO: u32 = 15;
const FONT_SCALE: gfx_glyph::Scale = gfx_glyph::Scale { x: 16.0, y: 16.0 };

const FONT_ID: gfx_glyph::FontId = gfx_glyph::FontId(0);
const BOLD_FONT_ID: gfx_glyph::FontId = gfx_glyph::FontId(1);

pub const MAX_WIDTH_IN_CELLS: u32 = 256;
pub const MAX_HEIGHT_IN_CELLS: u32 = 256;

#[derive(Debug, Clone, Copy)]
pub struct MonitorInfo {
    physical_size: PhysicalSize,
    hidpi_factor: f64,
}

impl MonitorInfo {
    fn from_monitor_id(monitor_id: glutin::MonitorId) -> Self {
        let physical_size = monitor_id.get_dimensions();
        let hidpi_factor = monitor_id.get_hidpi_factor();
        Self {
            physical_size,
            hidpi_factor,
        }
    }
    pub fn get_primary() -> Self {
        let events_loop = glutin::EventsLoop::new();
        let monitor_id = events_loop.get_primary_monitor();
        Self::from_monitor_id(monitor_id)
    }
    pub fn get_current() -> Self {
        let window_builder = glutin::WindowBuilder::new();
        let context_builder = glutin::ContextBuilder::new();
        let events_loop = glutin::EventsLoop::new();
        let (window, _device, _factory, _rtv, _dsv) =
            gfx_window_glutin::init::<ColourFormat, DepthFormat>(
                window_builder,
                context_builder,
                &events_loop,
            )
            .expect("Failed to create window");
        let monitor_id = window.get_current_monitor();
        Self::from_monitor_id(monitor_id)
    }
    pub fn physical_width(&self) -> f64 {
        self.physical_size.width
    }
    pub fn physical_height(&self) -> f64 {
        self.physical_size.height
    }
    pub fn logical_width(&self) -> f64 {
        self.physical_size.to_logical(self.hidpi_factor).width
    }
    pub fn logical_height(&self) -> f64 {
        self.physical_size.to_logical(self.hidpi_factor).height
    }
    pub fn hidpi_factor(&self) -> f64 {
        self.hidpi_factor
    }
}

#[derive(Debug)]
pub enum Error {
    GfxGlyph(String),
    GlutinContextError(glutin::ContextError),
    FailedToMeasureFont,
}
pub type Result<T> = ::std::result::Result<T, Error>;

pub struct ContextBuilder<'a> {
    font: &'a [u8],
    bold_font: Option<&'a [u8]>,
    font_scale: gfx_glyph::Scale,
    bold_font_scale: Option<gfx_glyph::Scale>,
    window_builder: glutin::WindowBuilder,
    context_builder: glutin::ContextBuilder<'a>,
    cell_dimensions: Option<Size>,
    underline_width: Option<u32>,
    underline_position: Option<u32>,
    max_grid_size: Option<Size>,
}

impl<'a> ContextBuilder<'a> {
    pub fn new_with_font(font: &'a [u8]) -> Self {
        Self {
            font,
            bold_font: None,
            font_scale: FONT_SCALE,
            bold_font_scale: None,
            window_builder: glutin::WindowBuilder::new(),
            context_builder: glutin::ContextBuilder::new(),
            cell_dimensions: None,
            underline_width: None,
            underline_position: None,
            max_grid_size: None,
        }
    }

    pub fn with_title<T: Into<String>>(self, title: T) -> Self {
        Self {
            window_builder: self.window_builder.with_title(title),
            ..self
        }
    }

    pub fn with_cell_dimensions(self, size: Size) -> Self {
        Self {
            cell_dimensions: Some(size),
            ..self
        }
    }

    pub fn with_window_dimensions(self, size: Size) -> Self {
        Self {
            window_builder: self.window_builder.with_dimensions(LogicalSize {
                width: size.width() as f64,
                height: size.height() as f64,
            }),
            ..self
        }
    }

    pub fn with_min_window_dimensions(self, size: Size) -> Self {
        Self {
            window_builder: self.window_builder.with_min_dimensions(LogicalSize {
                width: size.width() as f64,
                height: size.height() as f64,
            }),
            ..self
        }
    }

    pub fn with_max_window_dimensions(self, size: Size) -> Self {
        Self {
            window_builder: self.window_builder.with_max_dimensions(LogicalSize {
                width: size.width() as f64,
                height: size.height() as f64,
            }),
            ..self
        }
    }

    pub fn with_vsync(self, vsync: bool) -> Self {
        Self {
            context_builder: self.context_builder.with_vsync(vsync),
            ..self
        }
    }

    pub fn with_underline_width(self, underline_width: u32) -> Self {
        Self {
            underline_width: Some(underline_width),
            ..self
        }
    }

    pub fn with_underline_position(self, underline_position: u32) -> Self {
        Self {
            underline_position: Some(underline_position),
            ..self
        }
    }

    pub fn with_bold_font(self, bold_font: &'a [u8]) -> Self {
        Self {
            bold_font: Some(bold_font),
            ..self
        }
    }

    pub fn with_font_scale(self, x: f32, y: f32) -> Self {
        Self {
            font_scale: gfx_glyph::Scale { x, y },
            ..self
        }
    }

    pub fn with_bold_font_scale(self, x: f32, y: f32) -> Self {
        Self {
            bold_font_scale: Some(gfx_glyph::Scale { x, y }),
            ..self
        }
    }

    pub fn with_max_grid_size(self, size: Size) -> Self {
        Self {
            max_grid_size: Some(size),
            ..self
        }
    }

    pub fn build(self) -> Result<Context<'a>> {
        let events_loop = glutin::EventsLoop::new();

        let (window, device, mut factory, rtv, dsv) =
            gfx_window_glutin::init::<ColourFormat, DepthFormat>(
                self.window_builder,
                self.context_builder,
                &events_loop,
            )
            .expect("Failed to create window");
        let mut encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer().into();
        let mut glyph_brush_builder =
            gfx_glyph::GlyphBrushBuilder::using_font_bytes(self.font);
        let bold_font_id =
            glyph_brush_builder.add_font_bytes(self.bold_font.unwrap_or(self.font));
        assert_eq!(bold_font_id, BOLD_FONT_ID);
        let mut glyph_brush = glyph_brush_builder.build(factory.clone());
        let (window_width, window_height, _, _) = rtv.get_dimensions();
        let hidpi = window.window().get_hidpi_factor() as f32;
        let window_width = (window_width as f32 * hidpi) as u32;
        let window_height = (window_height as f32 * hidpi) as u32;
        let (cell_width, cell_height) =
            if let Some(cell_dimensions) = self.cell_dimensions {
                (cell_dimensions.x(), cell_dimensions.y())
            } else {
                let section = gfx_glyph::Section {
                    text: "@",
                    scale: self.font_scale,
                    font_id: FONT_ID,
                    ..Default::default()
                };
                let rect = glyph_brush
                    .pixel_bounds(section)
                    .ok_or(Error::FailedToMeasureFont)?;
                (rect.width() as u32, rect.height() as u32)
            };
        let unscaled_cell_width = cell_width as f32;
        let unscaled_cell_height = cell_height as f32;
        let cell_width = unscaled_cell_width * hidpi;
        let cell_height = unscaled_cell_height * hidpi;
        let max_grid_size = self
            .max_grid_size
            .unwrap_or(Size::new(MAX_WIDTH_IN_CELLS, MAX_HEIGHT_IN_CELLS));
        let width_in_cells =
            ::std::cmp::min((window_width as f32 / cell_width) as u32, max_grid_size.x());
        let height_in_cells = ::std::cmp::min(
            (window_height as f32 / cell_height) as u32,
            max_grid_size.y(),
        );
        let window_size_in_cells = Size::new(width_in_cells, height_in_cells);
        let grid = Grid::new(window_size_in_cells, GlutinColourConversion);
        let char_buf = String::with_capacity(1);
        let underline_width = self
            .underline_width
            .map(|w| hidpi * w as f32)
            .unwrap_or_else(|| cell_height as f32 / UNDERLINE_WIDTH_RATIO as f32);
        let underline_position = self
            .underline_position
            .map(|w| hidpi * w as f32)
            .unwrap_or_else(|| {
                cell_height as f32
                    - (cell_height as f32 / UNDERLINE_POSITION_RATIO as f32)
            });
        let background_renderer = BackgroundRenderer::new(
            window_width,
            window_height,
            unscaled_cell_width,
            unscaled_cell_height,
            window_size_in_cells,
            underline_width,
            underline_position,
            rtv.clone(),
            &mut factory,
            &mut encoder,
        );
        let font_scale = gfx_glyph::Scale {
            x: self.font_scale.x * hidpi,
            y: self.font_scale.y * hidpi,
        };
        let bold_font_scale = self
            .bold_font_scale
            .map(|s| gfx_glyph::Scale {
                x: s.x * hidpi,
                y: s.y * hidpi,
            })
            .unwrap_or(font_scale);
        Ok(Context {
            window,
            device,
            encoder,
            factory,
            rtv,
            dsv,
            events_loop,
            glyph_brush,
            grid,
            char_buf,
            cell_width,
            cell_height,
            unscaled_cell_width,
            unscaled_cell_height,
            max_grid_size,
            background_renderer,
            font_scale,
            bold_font_scale,
            closing: false,
            last_mouse_coord: Coord::new(0, 0),
            last_mouse_button: None,
        })
    }
}

struct GlutinColourConversion;
impl ColourConversion for GlutinColourConversion {
    type Colour = [f32; 4];
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        rgb24.to_f32_rgba(1.)
    }
    fn default_foreground(&mut self) -> Self::Colour {
        [1.0, 1.0, 1.0, 1.0]
    }
    fn default_background(&mut self) -> Self::Colour {
        [0.0, 0.0, 0.0, 1.0]
    }
}

pub struct Context<'a> {
    window: glutin::WindowedContext,
    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer>,
    factory: gfx_device_gl::Factory,
    rtv: gfx::handle::RenderTargetView<Resources, ColourFormat>,
    dsv: gfx::handle::DepthStencilView<Resources, DepthFormat>,
    events_loop: glutin::EventsLoop,
    glyph_brush: gfx_glyph::GlyphBrush<'a, Resources, gfx_device_gl::Factory>,
    grid: Grid<GlutinColourConversion>,
    char_buf: String,
    cell_width: f32,
    cell_height: f32,
    unscaled_cell_width: f32,
    unscaled_cell_height: f32,
    max_grid_size: Size,
    background_renderer: BackgroundRenderer<Resources>,
    font_scale: gfx_glyph::Scale,
    bold_font_scale: gfx_glyph::Scale,
    closing: bool,
    last_mouse_coord: Coord,
    last_mouse_button: Option<MouseButton>,
}

impl<'a> Context<'a> {
    pub fn poll_input<F: FnMut(Input)>(&mut self, mut callback: F) {
        let mut closing = false;
        let mut resize = None;
        let cell_dimensions = (self.unscaled_cell_width, self.unscaled_cell_height);
        let mut last_mouse_coord = self.last_mouse_coord;
        let mut last_mouse_button = self.last_mouse_button;
        self.events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => {
                    if let Some(event) = convert_event(
                        event,
                        cell_dimensions,
                        &mut last_mouse_coord,
                        &mut last_mouse_button,
                    ) {
                        match event {
                            InputEvent::Input(input) => callback(input),
                            InputEvent::Quit => closing = true,
                            InputEvent::Resize(width, height) => {
                                resize = Some((width, height))
                            }
                        }
                    }
                }
                _ => (),
            };
        });
        self.last_mouse_coord = last_mouse_coord;
        self.last_mouse_button = last_mouse_button;
        if let Some((width, height)) = resize {
            self.handle_resize(width, height);
        }
        if closing {
            self.closing = true;
            callback(inputs::ETX);
        }
    }

    pub fn buffer_input(&mut self, buffer: &mut Vec<Input>) {
        self.poll_input(|input| buffer.push(input));
    }

    fn handle_resize(&mut self, width: f32, height: f32) {
        let (rtv, dsv) = gfx_window_glutin::new_views(&self.window);
        let width_in_cells = ::std::cmp::min(
            (width as f32 / self.unscaled_cell_width) as u32,
            self.max_grid_size.x(),
        );
        let height_in_cells = ::std::cmp::min(
            (height as f32 / self.unscaled_cell_height) as u32,
            self.max_grid_size.y(),
        );
        let window_size_in_cells = Size::new(width_in_cells, height_in_cells);
        self.background_renderer.handle_resize(
            width,
            height,
            window_size_in_cells,
            rtv.clone(),
            &mut self.factory,
            &mut self.encoder,
        );
        self.rtv = rtv;
        self.dsv = dsv;
        self.grid.resize(window_size_in_cells);
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) -> Result<()> {
        let size = self.size();
        self.render_at(view, data, ViewContext::default_with_size(size))
    }

    pub fn render_at<V: View<T>, T, R: ViewTransformRgb24>(
        &mut self,
        view: &mut V,
        data: &T,
        context: ViewContext<R>,
    ) -> Result<()> {
        if self.closing {
            return Ok(());
        }
        self.grid.clear();
        view.view(data, context, &mut self.grid);
        {
            let mut output_cells = self.background_renderer.map_cells(&mut self.factory);
            for ((coord, cell), output_cell) in
                self.grid.enumerate().zip(output_cells.iter_mut())
            {
                self.char_buf.clear();
                self.char_buf.push(cell.character);
                let (font_id, scale) = if cell.bold {
                    (BOLD_FONT_ID, self.bold_font_scale)
                } else {
                    (FONT_ID, self.font_scale)
                };
                let section = gfx_glyph::Section {
                    text: self.char_buf.as_str(),
                    screen_position: (
                        (coord.x as f32 * self.cell_width),
                        (coord.y as f32 * self.cell_height),
                    ),
                    scale,
                    font_id,
                    color: cell.foreground_colour,
                    layout: Layout::default()
                        .h_align(HorizontalAlign::Left)
                        .v_align(VerticalAlign::Top),
                    ..Default::default()
                };
                self.glyph_brush.queue(section);
                output_cell.foreground_colour = cell.foreground_colour;
                output_cell.background_colour = cell.background_colour;
                output_cell.underline = cell.underline as u32;
            }
        }
        self.encoder.clear(&self.rtv, [0.0, 0.0, 0.0, 1.0]);
        self.background_renderer.draw(&mut self.encoder);
        self.glyph_brush
            .draw_queued(&mut self.encoder, &self.rtv, &self.dsv)
            .map_err(Error::GfxGlyph)?;
        self.encoder.flush(&mut self.device);
        self.window
            .swap_buffers()
            .map_err(Error::GlutinContextError)?;
        self.device.cleanup();
        Ok(())
    }

    pub fn size(&self) -> Size {
        self.grid.size()
    }
}
