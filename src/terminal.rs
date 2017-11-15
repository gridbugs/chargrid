use std::io::{self, Write};
use cgmath::Vector2;
use unix_backend::UnixBackend;
use term_info_strings::TermInfoStrings;
use term::terminfo::parm::{self, Variables, Param};
use error::Result;

const BUFFER_INITIAL_CAPACITY: usize = 32 * 1024;

pub struct Terminal {
    backend: UnixBackend,
    buffer: Vec<u8>,
    ti_strings: TermInfoStrings,
    ti_vars: Variables,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let backend = UnixBackend::new()?;
        let buffer = Vec::with_capacity(BUFFER_INITIAL_CAPACITY);
        let ti_strings = TermInfoStrings::new()?;
        let ti_vars = Variables::new();

        let mut terminal = Self {
            backend,
            buffer,
            ti_strings,
            ti_vars,
        };

        terminal.init()?;

        Ok(terminal)
    }

    fn init(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(&self.ti_strings.enter_ca);
        self.buffer.extend_from_slice(&self.ti_strings.enter_xmit);
        self.buffer.extend_from_slice(&self.ti_strings.hide_cursor);
        self.buffer.extend_from_slice(&self.ti_strings.clear);

        self.flush().map_err(Into::into)
    }

    fn reset(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(&self.ti_strings.exit_ca);
        self.buffer.extend_from_slice(&self.ti_strings.exit_xmit);
        self.buffer.extend_from_slice(&self.ti_strings.show_cursor);
        self.buffer.extend_from_slice(&self.ti_strings.reset);

        self.flush().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        self.backend.size()
    }

    pub fn move_cursor(&mut self, coord: Vector2<u16>) -> Result<()> {
        let params = &[Param::Number(coord.y as i32), Param::Number(coord.x as i32)];
        let command = parm::expand(&self.ti_strings.set_cursor, params, &mut self.ti_vars)?;
        self.buffer.extend_from_slice(&command);
        Ok(())
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.send(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.reset().expect("Failed to reset terminal to original settings");
    }
}
