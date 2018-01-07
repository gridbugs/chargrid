use gfx;
use glutin;
use gfx_device_gl;
use gfx_window_glutin;
use gfx_text;

use gfx::Device;
use glutin::{GlContext, Event, WindowEvent, VirtualKeyCode, ElementState};

use prototty_grid::*;
use prototty::*;
use cell::*;
use formats::*;
use background::*;

type Resources = gfx_device_gl::Resources;

const UNDERLINE_WIDTH_RATIO: u32 = 20;
const UNDERLINE_POSITION_RATIO: u32 = 20;

#[derive(Debug)]
pub enum Error {
    GfxText(gfx_text::Error),
    WindowNoLongerExists,
}
pub type Result<T> = ::std::result::Result<T, Error>;

enum Font<'a> {
    Path(&'a str),
    Data(&'a [u8]),
}

impl<'a> Font<'a> {
    fn set_builder_font(self, builder: gfx_text::RendererBuilder<'a, Resources, gfx_device_gl::Factory>)
        -> gfx_text::RendererBuilder<'a, Resources, gfx_device_gl::Factory>
    {
        match self {
            Font::Path(path) => builder.with_font(path),
            Font::Data(data) => builder.with_font_data(data),
        }
    }
}

pub struct ContextBuilder<'a> {
    font: Option<Font<'a>>,
    bold_font: Option<Font<'a>>,
    font_size: Option<u8>,
    bold_font_size: Option<u8>,
    window_builder: glutin::WindowBuilder,
    context_builder: glutin::ContextBuilder<'a>,
    cell_dimensions: Option<Size>,
    locked_size: bool,
    underline_width: Option<u32>,
    underline_position: Option<u32>,
}

impl<'a> ContextBuilder<'a> {
    pub fn new() -> Self {
        Self {
            font: None,
            bold_font: None,
            font_size: None,
            bold_font_size: None,
            window_builder: glutin::WindowBuilder::new(),
            context_builder: glutin::ContextBuilder::new(),
            cell_dimensions: None,
            locked_size: false,
            underline_width: None,
            underline_position: None,
        }
    }

    pub fn with_cell_dimensions(self, width: u32, height: u32) -> Self {
        Self {
            cell_dimensions: Some(Size::new(width, height)),
            ..self
        }
    }

    pub fn with_window_dimensions(self, width: u32, height: u32) -> Self {
        Self {
            window_builder: self.window_builder.with_dimensions(width, height),
            ..self
        }
    }

    pub fn with_vsync(self, vsync: bool) -> Self {
        Self {
            context_builder: self.context_builder.with_vsync(vsync),
            ..self
        }
    }

    pub fn with_locked_size(self, locked_size: bool) -> Self {
        Self {
            locked_size,
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

    pub fn with_font_path(self, path: &'a str) -> Self {
        Self {
            font: Some(Font::Path(path)),
            ..self
        }
    }

    pub fn with_bold_font_path(self, path: &'a str) -> Self {
        Self {
            bold_font: Some(Font::Path(path)),
            ..self
        }
    }

    pub fn with_font_data(self, data: &'a [u8]) -> Self {
        Self {
            font: Some(Font::Data(data)),
            ..self
        }
    }

    pub fn with_bold_font_data(self, data: &'a [u8]) -> Self {
        Self {
            bold_font: Some(Font::Data(data)),
            ..self
        }
    }

    pub fn with_font_size(self, size: u8) -> Self {
        Self {
            font_size: Some(size),
            ..self
        }
    }

    pub fn with_bold_font_size(self, size: u8) -> Self {
        Self {
            bold_font_size: Some(size),
            ..self
        }
    }

    pub fn build(self) -> Result<Context> {

        let events_loop = glutin::EventsLoop::new();

        let (window, device, mut factory, rtv, _dsv) =
            gfx_window_glutin::init::<ColourFormat, DepthFormat>(self.window_builder, self.context_builder, &events_loop);

        let mut encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer().into();

        let mut renderer_builder = gfx_text::new(factory.clone());
        let mut bold_renderer_builder = gfx_text::new(factory.clone());

        if let Some(font) = self.font {
            renderer_builder = font.set_builder_font(renderer_builder);
        }
        if let Some(font) = self.bold_font {
            bold_renderer_builder = font.set_builder_font(bold_renderer_builder);
        }

        if let Some(font_size) = self.font_size {
            renderer_builder = renderer_builder.with_size(font_size);
        }
        if let Some(font_size) = self.bold_font_size {
            bold_renderer_builder = bold_renderer_builder.with_size(font_size);
        }

        let text_renderer = renderer_builder.build().map_err(Error::GfxText)?;
        let bold_text_renderer = bold_renderer_builder.build().map_err(Error::GfxText)?;

        let (window_width, window_height) = window.get_outer_size().ok_or(Error::WindowNoLongerExists)?;

        let (cell_width, cell_height) = if let Some(cell_dimensions) = self.cell_dimensions {
            (cell_dimensions.x(), cell_dimensions.y())
        } else {
            let (width, height) = text_renderer.measure("@");
            (width as u32, height as u32)
        };

        let width_in_cells = window_width / cell_width;
        let height_in_cells = window_height / cell_height;

        let window_size_in_cells = Size::new(width_in_cells, height_in_cells);

        let grid = Grid::new(window_size_in_cells);
        let char_buf = String::with_capacity(1);

        let underline_width = self.underline_width.unwrap_or_else(|| {
            cell_height / UNDERLINE_WIDTH_RATIO
        });

        let underline_position = self.underline_position.unwrap_or_else(|| {
            cell_height - (cell_height / UNDERLINE_POSITION_RATIO)
        });

        let background_renderer = BackgroundRenderer::new(
            window_width,
            window_height,
            cell_width,
            cell_height,
            window_size_in_cells,
            rtv.clone(),
            &mut factory,
            &mut encoder);

        Ok(Context {
            window,
            device,
            encoder,
            factory,
            rtv,
            events_loop,
            text_renderer,
            bold_text_renderer,
            grid,
            char_buf,
            cell_width,
            cell_height,
            locked_size: self.locked_size,
            underline_width,
            underline_position,
            background_renderer,
        })
    }
}

pub struct Context {
    window: glutin::GlWindow,
    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer>,
    factory: gfx_device_gl::Factory,
    rtv: gfx::handle::RenderTargetView<Resources, ColourFormat>,
    events_loop: glutin::EventsLoop,
    text_renderer: gfx_text::Renderer<Resources, gfx_device_gl::Factory>,
    bold_text_renderer: gfx_text::Renderer<Resources, gfx_device_gl::Factory>,
    grid: Grid<Cell>,
    char_buf: String,
    cell_width: u32,
    cell_height: u32,
    locked_size: bool,
    underline_width: u32,
    underline_position: u32,
    background_renderer: BackgroundRenderer<Resources>,
}

impl Context {
    pub fn poll_input<F: FnMut(Input)>(&mut self, mut callback: F) {
        self.events_loop.poll_events(|event| {
            let input = match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Closed => inputs::ETX,
                        WindowEvent::KeyboardInput { input, .. } => {
                            if let ElementState::Pressed = input.state {
                                if let Some(virtual_keycode) = input.virtual_keycode {
                                    match virtual_keycode {
                                        VirtualKeyCode::Left => Input::Left,
                                        VirtualKeyCode::Right => Input::Right,
                                        VirtualKeyCode::Up => Input::Up,
                                        VirtualKeyCode::Down => Input::Down,
                                        VirtualKeyCode::Escape => inputs::ESCAPE,
                                        VirtualKeyCode::Return => inputs::RETURN,
                                        _ => return,
                                    }
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        }
                        _ => return,
                    }
                }
                _ => return,
            };

            callback(input);
        });
    }
}

impl Renderer for Context {
    type Error = Error;
    fn render<V: View<T>, T>(&mut self, view: &V, data: &T) -> Result<()> {

        self.grid.clear();
        view.view(data, Coord::new(0, 0), 0, &mut self.grid);

        {
            let mut output_cells = self.background_renderer.map_cells(&mut self.factory);

            for ((coord, cell), output_cell) in izip!(self.grid.enumerate(), output_cells.iter_mut()) {
                self.char_buf.clear();
                self.char_buf.push(cell.character);

                if cell.bold {
                    &mut self.bold_text_renderer
                } else {
                    &mut self.text_renderer
                }.add(self.char_buf.as_str(),
                      [coord.x * self.cell_width as i32, coord.y * self.cell_height as i32],
                      cell.foreground_colour.0);

                output_cell.foreground_colour = cell.foreground_colour.0;
                output_cell.background_colour = cell.background_colour.0;
                output_cell.underline = cell.underline as u32;
            }
        }

        self.encoder.clear(&self.rtv, [0.0,0.0,0.0,1.0]);

        self.background_renderer.draw(&mut self.encoder);
        self.bold_text_renderer.draw(&mut self.encoder, &self.rtv).unwrap();
        self.text_renderer.draw(&mut self.encoder, &self.rtv).unwrap();

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().expect("Failed to swap buffers");
        self.device.cleanup();

        Ok(())
    }
}
