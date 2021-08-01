use crate::error::*;
use crate::terminal::*;
#[cfg(feature = "gamepad")]
use chargrid_gamepad::GamepadContext;
use chargrid_runtime::{app, on_frame, on_input, Component, FrameBuffer, Size};
use std::thread;
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

/// An interface to a terminal for rendering `View`s, and getting input.
pub struct Context {
    terminal: Terminal,
    chargrid_frame_buffer: FrameBuffer,
    #[cfg(feature = "gamepad")]
    gamepad: GamepadContext,
}

impl Context {
    /// Initialise a new context using the current terminal.
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    fn from_terminal(mut terminal: Terminal) -> Result<Self> {
        let size = terminal.resize_if_necessary()?;
        let chargrid_frame_buffer = FrameBuffer::new(size);
        Ok(Self {
            terminal,
            chargrid_frame_buffer,
            #[cfg(feature = "gamepad")]
            gamepad: GamepadContext::new(),
        })
    }

    pub fn run_component<C, E>(self, mut component: C, col_encode: E)
    where
        C: 'static + Component<State = (), Output = app::Output>,
        E: ColEncode,
    {
        let _ = col_encode;
        let Self {
            mut terminal,
            mut chargrid_frame_buffer,
            #[cfg(feature = "gamepad")]
            mut gamepad,
        } = self;
        loop {
            let frame_start = Instant::now();
            for input in terminal.drain_input().unwrap() {
                if let Some(app::Exit) = on_input(&mut component, input, &chargrid_frame_buffer) {
                    return;
                }
            }
            #[cfg(feature = "gamepad")]
            for input in gamepad.drain_input() {
                if let Some(app::Exit) = on_input(
                    &mut component,
                    chargrid_input::Input::Gamepad(input),
                    &chargrid_frame_buffer,
                ) {
                    return;
                }
            }
            let terminal_size = terminal.resize_if_necessary().unwrap();
            if terminal_size != chargrid_frame_buffer.size() {
                chargrid_frame_buffer.resize(terminal_size);
            }
            if let Some(app::Exit) =
                on_frame(&mut component, FRAME_DURATION, &mut chargrid_frame_buffer)
            {
                return;
            }
            terminal
                .draw_frame::<E>(&mut chargrid_frame_buffer)
                .unwrap();
            let since_frame_start = frame_start.elapsed();
            if let Some(until_next_frame) = FRAME_DURATION.checked_sub(since_frame_start) {
                thread::sleep(until_next_frame);
            }
        }
    }

    pub fn size(&self) -> Result<Size> {
        self.terminal.size()
    }
}
