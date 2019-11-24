use crate::error::*;
use crate::terminal::*;
use prototty_app::{App, ControlFlow};
use prototty_event_routine::{Event, EventRoutine, Handled};
use prototty_input::*;
use prototty_render::*;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

/// An interface to a terminal for rendering `View`s, and getting input.
pub struct Context {
    terminal: Terminal,
    buffer: Buffer,
}

impl Context {
    /// Initialise a new context using the current terminal.
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    pub fn from_terminal(mut terminal: Terminal) -> Result<Self> {
        let size = terminal.resize_if_necessary()?;
        let buffer = Buffer::new(size);
        Ok(Self { terminal, buffer })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.resize_if_necessary()?;
        if size != self.buffer.size() {
            self.buffer.resize(size);
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

    pub fn run_app<A, E>(mut self, mut app: A, col_encode: E)
    where
        A: App + 'static,
        E: ColEncode,
    {
        let _ = col_encode;
        loop {
            for input in self.drain_input().unwrap() {
                if let Some(ControlFlow::Exit) = app.on_input(input) {
                    return;
                }
            }
            self.resize_if_necessary().unwrap();
            self.buffer.clear();
            let view_context = ViewContext::default_with_size(self.size().unwrap());
            if let Some(ControlFlow::Exit) = app.on_frame(FRAME_DURATION, view_context, &mut self.buffer) {
                return;
            }
            self.terminal.draw_frame::<E>(&mut self.buffer).unwrap();
            thread::sleep(FRAME_DURATION);
        }
    }

    fn render_internal<E>(&mut self, col_encode: E) -> Result<()>
    where
        E: ColEncode,
    {
        let _ = col_encode;
        self.terminal.draw_frame::<E>(&mut self.buffer)
    }

    pub fn render<V, T, E>(&mut self, view: &mut V, data: T, col_encode: E) -> Result<()>
    where
        V: View<T>,
        E: ColEncode,
    {
        let size = self.size()?;
        self.render_at(view, data, ViewContext::default_with_size(size), col_encode)
    }

    pub fn render_at<V, T, C, E>(&mut self, view: &mut V, data: T, context: ViewContext<C>, col_encode: E) -> Result<()>
    where
        V: View<T>,
        C: ColModify,
        E: ColEncode,
    {
        self.resize_if_necessary()?;
        self.buffer.clear();
        view.view(data, context, &mut self.buffer);
        self.render_internal(col_encode)
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
        self.buffer.clear();
        Ok(UnixFrame { context: self })
    }
}

pub struct UnixFrame<'a> {
    context: &'a mut Context,
}

impl<'a> UnixFrame<'a> {
    pub fn render<E>(self, col_encode: E) -> Result<()>
    where
        E: ColEncode,
    {
        self.context.render_internal(col_encode)
    }

    fn size(&self) -> Size {
        self.context.buffer.size()
    }

    pub fn default_context(&self) -> ViewContextDefault {
        ViewContext::default_with_size(self.size())
    }
}

impl<'a> Frame for UnixFrame<'a> {
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i8, absolute_cell: ViewCell) {
        self.context
            .buffer
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
    fn blend_cell_background_absolute<B: Blend>(
        &mut self,
        absolute_coord: Coord,
        absolute_depth: i8,
        rgb24: Rgb24,
        alpha: u8,
        blend: B,
    ) {
        self.context
            .buffer
            .blend_cell_background_absolute(absolute_coord, absolute_depth, rgb24, alpha, blend);
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
        col_encode: EC,
    ) -> Result<ER::Return>
    where
        ER: EventRoutine<Event = V>,
        V: From<Input> + From<Duration>,
        EC: ColEncode,
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
            frame.render(col_encode.clone())?;
            if let Some(time_until_next_frame) = self.frame_duration.checked_sub(frame_instant.elapsed()) {
                thread::sleep(time_until_next_frame);
            }
        }
    }
}
