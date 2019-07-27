pub use ansi_colour::Colour as AnsiColour;
use error::*;
use prototty_event_routine::{Event, EventRoutine, Handled};
use prototty_grid::*;
use prototty_input::*;
use prototty_render::*;
use std::thread;
use std::time::{Duration, Instant};
use terminal::*;

const DEFAULT_FG: AnsiColour = AnsiColour::from_rgb24(grey24(255));
const DEFAULT_BG: AnsiColour = AnsiColour::from_rgb24(grey24(0));

struct UnixColourConversion;

impl ColourConversion for UnixColourConversion {
    type Colour = AnsiColour;
    fn default_foreground(&mut self) -> Self::Colour {
        DEFAULT_FG
    }
    fn default_background(&mut self) -> Self::Colour {
        DEFAULT_BG
    }
    fn convert_foreground_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        AnsiColour::from_rgb24(rgb24)
    }
    fn convert_background_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        AnsiColour::from_rgb24(rgb24)
    }
}

/// An interface to a terminal for rendering `View`s, and getting input.
pub struct Context {
    terminal: Terminal,
    frame: Grid<UnixColourConversion>,
}

impl Context {
    /// Initialise a new context using the current terminal.
    pub fn new() -> Result<Self> {
        Terminal::new(DEFAULT_FG, DEFAULT_BG).and_then(Self::from_terminal)
    }

    pub fn from_terminal(mut terminal: Terminal) -> Result<Self> {
        let size = terminal.resize_if_necessary(DEFAULT_FG, DEFAULT_BG)?;
        let frame = Grid::new(size, UnixColourConversion);
        Ok(Self { terminal, frame })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self
            .terminal
            .resize_if_necessary(self.frame.default_foreground(), self.frame.default_background())?;
        if size != self.frame.size() {
            self.frame.resize(size);
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

    fn render_internal<E>(&mut self, encode_colour: E) -> Result<()>
    where
        E: EncodeColour,
    {
        let _ = encode_colour;
        self.terminal.draw_frame::<_, E>(&mut self.frame)
    }

    pub fn render<V, T, E>(&mut self, view: &mut V, data: T, encode_colour: E) -> Result<()>
    where
        V: View<T>,
        E: EncodeColour,
    {
        let size = self.size()?;
        self.render_at(view, data, ViewContext::default_with_size(size), encode_colour)
    }

    pub fn render_at<V, T, R, E>(
        &mut self,
        view: &mut V,
        data: T,
        context: ViewContext<R>,
        encode_colour: E,
    ) -> Result<()>
    where
        V: View<T>,
        R: ViewTransformRgb24,
        E: EncodeColour,
    {
        self.resize_if_necessary()?;
        self.frame.clear();
        view.view(data, context, &mut self.frame);
        self.render_internal(encode_colour)
    }

    pub fn size(&self) -> Result<Size> {
        self.terminal.size()
    }

    pub fn into_runner(self, frame_duration: Duration) -> EventRoutineRunner {
        EventRoutineRunner {
            context: self,
            frame_duration,
        }
    }

    pub fn frame(&mut self) -> Result<UnixFrame> {
        self.resize_if_necessary()?;
        self.frame.clear();
        Ok(UnixFrame { context: self })
    }
}

pub struct UnixFrame<'a> {
    context: &'a mut Context,
}

impl<'a> UnixFrame<'a> {
    pub fn render<E>(self, encode_colour: E) -> Result<()>
    where
        E: EncodeColour,
    {
        self.context.render_internal(encode_colour)
    }
}

impl<'a> Frame for UnixFrame<'a> {
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i32, absolute_cell: ViewCell) {
        self.context
            .frame
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }

    fn size(&self) -> Size {
        self.context.frame.size()
    }
}

pub struct EventRoutineRunner {
    context: Context,
    frame_duration: Duration,
}

impl EventRoutineRunner {
    pub fn run<ER, V, EC>(
        &mut self,
        mut event_routine: ER,
        data: &mut ER::Data,
        view: &mut ER::View,
        encode_colour: EC,
    ) -> Result<ER::Return>
    where
        ER: EventRoutine<Event = V>,
        V: From<Input> + From<Duration>,
        EC: EncodeColour + Copy,
    {
        let mut frame_instant = Instant::now();
        loop {
            let duration = frame_instant.elapsed();
            frame_instant = Instant::now();
            for input in self.context.drain_input()? {
                event_routine = match event_routine.handle(data, view, Event::new(input.into())) {
                    Handled::Continue(event_routine) => event_routine,
                    Handled::Return(r) => return Ok(r),
                }
            }
            event_routine = match event_routine.handle(data, view, Event::new(duration.into())) {
                Handled::Continue(event_routine) => event_routine,
                Handled::Return(r) => return Ok(r),
            };
            let mut frame = self.context.frame()?;
            event_routine.view(data, view, frame.default_context(), &mut frame);
            frame.render(encode_colour)?;
            if let Some(time_until_next_frame) = self.frame_duration.checked_sub(frame_instant.elapsed()) {
                thread::sleep(time_until_next_frame);
            }
        }
    }
}
