mod background;
mod formats;
mod gfx_context;
mod input;

use prototty_app as app;
pub use prototty_graphical_common::*;
use prototty_render::Size;

pub struct Context {
    events_loop: glutin::EventsLoop,
    gfx_context: gfx_context::Context<'static>,
}

pub type ContextBuildError = gfx_context::Error;

impl Context {
    pub fn new(
        ContextDescriptor {
            font_bytes,
            title,
            window_dimensions,
            cell_dimensions,
            font_dimensions,
            underline_width,
            underline_top_offset,
        }: ContextDescriptor,
    ) -> Result<Self, ContextBuildError> {
        let normal_font: &'static [u8] = Box::leak(font_bytes.normal.into_boxed_slice());
        let bold_font: &'static [u8] = Box::leak(font_bytes.bold.into_boxed_slice());
        let builder = gfx_context::ContextBuilder::new_with_font(normal_font)
            .with_bold_font(bold_font)
            .with_title(title)
            .with_underline_width((underline_width * cell_dimensions.height) as u32)
            .with_underline_position((underline_top_offset * cell_dimensions.height) as u32)
            .with_cell_dimensions(Size::new(cell_dimensions.width as u32, cell_dimensions.height as u32))
            .with_font_scale(font_dimensions.width as f32, font_dimensions.height as f32);
        let builder = match window_dimensions {
            WindowDimensions::Fullscreen => builder.with_fullscreen(),
            WindowDimensions::Windowed(Dimensions { width, height }) => {
                let size = Size::new(width as u32, height as u32);
                builder
                    .with_window_dimensions(size)
                    .with_min_window_dimensions(size)
                    .with_max_window_dimensions(size)
            }
        };
        let (gfx_context, events_loop) = builder.build()?;
        Ok(Self {
            gfx_context,
            events_loop,
        })
    }
    pub fn run_app<A>(self, app: A)
    where
        A: app::App + 'static,
    {
        self.gfx_context.run_app(app, self.events_loop)
    }
}
