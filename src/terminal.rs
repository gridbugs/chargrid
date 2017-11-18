use std::io::{self, Write};
use std::time::Duration;
use cgmath::Vector2;
use terminal_colour::Colour;
use unix_backend::UnixBackend;
use term_info_cache::TermInfoCache;
use term::terminfo::parm::{self, Param};
use error::{Result, Error};
use input::Input;

const OUTPUT_BUFFER_INITIAL_CAPACITY: usize = 32 * 1024;
const INPUT_BUFFER_INITIAL_CAPACITY: usize = 32;

const ESCAPE: u8 = 0x1b;
const ESCAPE_CHAR: char = '\u{1b}';

pub struct Terminal {
    backend: UnixBackend,
    output_buffer: Vec<u8>,
    input_buffer: Vec<u8>,
    ti_cache: TermInfoCache,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let backend = UnixBackend::new()?;
        let output_buffer = Vec::with_capacity(OUTPUT_BUFFER_INITIAL_CAPACITY);
        let input_buffer = Vec::with_capacity(INPUT_BUFFER_INITIAL_CAPACITY);
        let ti_cache = TermInfoCache::new()?;

        let mut terminal = Self {
            backend,
            output_buffer,
            input_buffer,
            ti_cache,
        };

        terminal.init()?;

        Ok(terminal)
    }

    fn init(&mut self) -> Result<()> {
        self.output_buffer.extend_from_slice(&self.ti_cache.enter_ca);
        self.output_buffer.extend_from_slice(&self.ti_cache.enter_xmit);
        self.output_buffer.extend_from_slice(&self.ti_cache.hide_cursor);
        self.output_buffer.extend_from_slice(&self.ti_cache.clear);

        self.flush().map_err(Into::into)
    }

    fn teardown(&mut self) -> Result<()> {
        self.output_buffer.extend_from_slice(&self.ti_cache.exit_ca);
        self.output_buffer.extend_from_slice(&self.ti_cache.exit_xmit);
        self.output_buffer.extend_from_slice(&self.ti_cache.show_cursor);
        self.output_buffer.extend_from_slice(&self.ti_cache.reset);

        self.flush().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        self.backend.size()
    }

    pub fn set_cursor(&mut self, coord: Vector2<u16>) -> Result<()> {
        let params = &[Param::Number(coord.y as i32), Param::Number(coord.x as i32)];
        let command = parm::expand(&self.ti_cache.set_cursor, params, &mut self.ti_cache.vars)?;
        self.output_buffer.extend_from_slice(&command);
        Ok(())
    }

    pub fn set_foreground(&mut self, colour: Colour) {
        self.output_buffer.extend_from_slice(self.ti_cache.fg_colour(colour));
    }

    pub fn set_background(&mut self, colour: Colour) {
        self.output_buffer.extend_from_slice(self.ti_cache.bg_colour(colour));
    }

    pub fn set_bold(&mut self) {
        self.output_buffer.extend_from_slice(&self.ti_cache.bold);
    }

    pub fn set_underline(&mut self) {
        self.output_buffer.extend_from_slice(&self.ti_cache.underline);
    }

    pub fn reset(&mut self) {
        self.output_buffer.extend_from_slice(&self.ti_cache.reset);
    }

    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.backend.read_polling(&mut self.input_buffer)?;
        self.drain_input()
    }

    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.backend.read_timeout(&mut self.input_buffer, timeout)?;
        self.drain_input()
    }

    fn drain_input(&mut self) -> Result<Option<Input>> {
        let input = self.input_in_buffer()?;
        self.input_buffer.clear();
        Ok(input)
    }

    fn input_in_buffer(&self) -> Result<Option<Input>> {
        if let Some(first) = self.input_buffer.first() {
            if *first == ESCAPE {
                if self.input_buffer.len() == 1 {
                    // buffer contains literal escape character
                    Ok(Some(Input::Char(ESCAPE_CHAR)))
                } else {
                    // assume buffer contains escape sequence
                    let input = self.ti_cache.escape_sequences
                        .get(&self.input_buffer);
                    if input.is_none() {
                        Err(Error::UnrecognizedEscapeSequence(self.input_buffer.clone()))
                    } else {
                        Ok(input.cloned())
                    }
                }
            } else {
                // buffer contains characters - return the first,
                // ignoring the rest
                let s = ::std::str::from_utf8(&self.input_buffer)?;
                Ok(s.chars().next().map(Input::Char))
            }
        } else {
            // buffer is empty
            Ok(None)
        }
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.send(&self.output_buffer)?;
        self.output_buffer.clear();
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.teardown().expect("Failed to reset terminal to original settings");
    }
}
