use std::io::{self, Write};
use cgmath::Vector2;
use terminal_colour::Colour;
use unix_backend::UnixBackend;
use term_info_cache::TermInfoCache;
use term::terminfo::parm::{self, Param};
use error::Result;

const BUFFER_INITIAL_CAPACITY: usize = 32 * 1024;

pub struct Terminal {
    backend: UnixBackend,
    buffer: Vec<u8>,
    ti_cache: TermInfoCache,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let backend = UnixBackend::new()?;
        let buffer = Vec::with_capacity(BUFFER_INITIAL_CAPACITY);
        let ti_cache = TermInfoCache::new()?;

        let mut terminal = Self {
            backend,
            buffer,
            ti_cache,
        };

        terminal.init()?;

        Ok(terminal)
    }

    fn init(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(&self.ti_cache.enter_ca);
        self.buffer.extend_from_slice(&self.ti_cache.enter_xmit);
        self.buffer.extend_from_slice(&self.ti_cache.hide_cursor);
        self.buffer.extend_from_slice(&self.ti_cache.clear);

        self.flush().map_err(Into::into)
    }

    fn teardown(&mut self) -> Result<()> {
        self.buffer.extend_from_slice(&self.ti_cache.exit_ca);
        self.buffer.extend_from_slice(&self.ti_cache.exit_xmit);
        self.buffer.extend_from_slice(&self.ti_cache.show_cursor);
        self.buffer.extend_from_slice(&self.ti_cache.reset);

        self.flush().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        self.backend.size()
    }

    pub fn set_cursor(&mut self, coord: Vector2<u16>) -> Result<()> {
        let params = &[Param::Number(coord.y as i32), Param::Number(coord.x as i32)];
        let command = parm::expand(&self.ti_cache.set_cursor, params, &mut self.ti_cache.vars)?;
        self.buffer.extend_from_slice(&command);
        Ok(())
    }

    pub fn set_foreground(&mut self, colour: Colour) {
        self.buffer.extend_from_slice(self.ti_cache.fg_colour(colour));
    }

    pub fn set_background(&mut self, colour: Colour) {
        self.buffer.extend_from_slice(self.ti_cache.bg_colour(colour));
    }

    pub fn set_bold(&mut self) {
        self.buffer.extend_from_slice(&self.ti_cache.bold);
    }

    pub fn set_underline(&mut self) {
        self.buffer.extend_from_slice(&self.ti_cache.underline);
    }

    pub fn reset(&mut self) {
        self.buffer.extend_from_slice(&self.ti_cache.reset);
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
        self.teardown().expect("Failed to reset terminal to original settings");
    }
}
