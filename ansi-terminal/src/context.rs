use crate::error::*;
use crate::terminal::*;
use prototty_app::{App, ControlFlow};
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
            let frame_start = Instant::now();
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
            let since_frame_start = frame_start.elapsed();
            eprintln!("{:?}", since_frame_start);
            if let Some(until_next_frame) = FRAME_DURATION.checked_sub(since_frame_start) {
                thread::sleep(until_next_frame);
            }
        }
    }

    pub fn size(&self) -> Result<Size> {
        self.terminal.size()
    }
}
