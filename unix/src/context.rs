pub use ansi_colour::Colour as AnsiColour;
use error::*;
use prototty_grid::*;
use prototty_input::*;
use prototty_render::*;
use std::time::Duration;
use terminal::*;

pub trait ConvertRgb24 {
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> AnsiColour;
}

pub struct DefaultConvertRgb24;

impl ConvertRgb24 for DefaultConvertRgb24 {
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> AnsiColour {
        AnsiColour::from_rgb24(rgb24)
    }
}

impl<F: FnMut(Rgb24) -> AnsiColour> ConvertRgb24 for F {
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> AnsiColour {
        (self)(rgb24)
    }
}

struct UnixColourConversion<C>(C);

impl<C: ConvertRgb24> ColourConversion for UnixColourConversion<C> {
    type Colour = AnsiColour;
    fn default_foreground(&mut self) -> Self::Colour {
        AnsiColour::from_rgb24(grey24(255))
    }
    fn default_background(&mut self) -> Self::Colour {
        AnsiColour::from_rgb24(grey24(0))
    }
    fn convert_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        self.0.convert_rgb24(rgb24)
    }
}

/// An interface to a terminal for rendering `View`s, and getting input.
pub struct Context<C: ConvertRgb24 = DefaultConvertRgb24> {
    terminal: Terminal,
    grid: Grid<UnixColourConversion<C>>,
}

impl Context<DefaultConvertRgb24> {
    /// Initialise a new context using the current terminal.
    pub fn new() -> Result<Self> {
        Self::with_convert_rgb24(DefaultConvertRgb24)
    }

    pub fn from_terminal(terminal: Terminal) -> Result<Self> {
        Self::from_terminal_with_convert_rgb24(terminal, DefaultConvertRgb24)
    }
}

impl<C: ConvertRgb24> Context<C> {
    pub fn with_convert_rgb24(convert_rgb24: C) -> Result<Self> {
        Terminal::new().and_then(|terminal| {
            Self::from_terminal_with_convert_rgb24(terminal, convert_rgb24)
        })
    }

    pub fn from_terminal_with_convert_rgb24(
        mut terminal: Terminal,
        convert_rgb24: C,
    ) -> Result<Self> {
        let size = terminal.resize_if_necessary()?;
        let grid = Grid::new(size, UnixColourConversion(convert_rgb24));
        Ok(Self { terminal, grid })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.resize_if_necessary()?;
        if size != self.grid.size() {
            self.grid.resize(size);
        }
        Ok(())
    }

    pub fn drain_input(&mut self) -> Result<DrainInput> {
        self.terminal.drain_input()
    }

    /// Gets an input event from the terminal if one is present,
    /// returning immediately.
    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.terminal.poll_input()
    }

    /// Gets an input event from the terminal, waiting until
    /// an event occurs.
    pub fn wait_input(&mut self) -> Result<Input> {
        self.terminal.wait_input()
    }

    /// Gets an input event from the terminal, waiting until
    /// either an event occurs, or the timeout expires, in which
    /// case this method returns `None`.
    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.terminal.wait_input_timeout(timeout)
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) -> Result<()> {
        self.render_at(view, data, Default::default())
    }

    pub fn render_at<V: View<T>, T, R: ViewTransformRgb24>(
        &mut self,
        view: &mut V,
        data: &T,
        context: ViewContext<R>,
    ) -> Result<()> {
        self.resize_if_necessary()?;
        self.grid.clear();
        view.view(data, context, &mut self.grid);
        self.terminal.draw_grid(&self.grid)
    }

    pub fn size(&self) -> Result<Size> {
        self.terminal.size()
    }
}
