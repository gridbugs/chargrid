use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::Duration;
use cgmath::Vector2;
use terminal_colour::Colour;
use unix_backend::UnixBackend;
use term_info_cache::TermInfoCache;
use term::terminfo::parm::{self, Param};
use error::Result;
use input::Input;
use byte_prefix_tree::{BytePrefixTree, Found};

const OUTPUT_BUFFER_INITIAL_CAPACITY: usize = 32 * 1024;
const INPUT_BUFFER_INITIAL_CAPACITY: usize = 32;
const INPUT_RING_INITIAL_CAPACITY: usize = 32;

pub struct Terminal {
    backend: UnixBackend,
    output_buffer: String,
    input_buffer: Vec<u8>,
    ti_cache: TermInfoCache,
    input_ring: VecDeque<Input>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let backend = UnixBackend::new()?;
        let output_buffer = String::with_capacity(OUTPUT_BUFFER_INITIAL_CAPACITY);
        let input_buffer = Vec::with_capacity(INPUT_BUFFER_INITIAL_CAPACITY);
        let ti_cache = TermInfoCache::new()?;
        let input_ring = VecDeque::with_capacity(INPUT_RING_INITIAL_CAPACITY);

        let mut terminal = Self {
            backend,
            output_buffer,
            input_buffer,
            ti_cache,
            input_ring,
        };

        terminal.init()?;

        Ok(terminal)
    }

    fn init(&mut self) -> Result<()> {
        self.output_buffer.push_str(&self.ti_cache.enter_ca);
        self.output_buffer.push_str(&self.ti_cache.enter_xmit);
        self.output_buffer.push_str(&self.ti_cache.hide_cursor);
        self.output_buffer.push_str(&self.ti_cache.clear);

        self.flush().map_err(Into::into)
    }

    fn teardown(&mut self) -> Result<()> {
        self.output_buffer.push_str(&self.ti_cache.exit_ca);
        self.output_buffer.push_str(&self.ti_cache.exit_xmit);
        self.output_buffer.push_str(&self.ti_cache.show_cursor);
        self.output_buffer.push_str(&self.ti_cache.reset);

        self.flush().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        self.backend.size()
    }

    pub fn set_cursor(&mut self, coord: Vector2<u16>) -> Result<()> {
        let params = &[Param::Number(coord.y as i32), Param::Number(coord.x as i32)];
        let command = parm::expand(self.ti_cache.set_cursor.as_bytes(), params, &mut self.ti_cache.vars)?;
        let command_slice = ::std::str::from_utf8(&command)?;
        self.output_buffer.push_str(command_slice);
        Ok(())
    }

    pub fn set_foreground(&mut self, colour: Colour) {
        self.output_buffer.push_str(self.ti_cache.fg_colour(colour));
    }

    pub fn set_background(&mut self, colour: Colour) {
        self.output_buffer.push_str(self.ti_cache.bg_colour(colour));
    }

    pub fn set_bold(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.bold);
    }

    pub fn set_underline(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.underline);
    }

    pub fn reset(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.reset);
    }

    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        if let Some(input) = self.input_ring.pop_front() {
            return Ok(Some(input));
        }
        self.backend.read_polling(&mut self.input_buffer)?;
        self.drain_input_into_ring()?;
        Ok(self.input_ring.pop_front())
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        let input = loop {
            // This loop is for good measure. If for some reason
            // read_waiting returns without a new input becoming
            // available, this will ensure we don't return without
            // an input.
            self.backend.read_waiting(&mut self.input_buffer)?;
            self.drain_input_into_ring()?;
            if let Some(input) = self.input_ring.pop_front() {
                break input;
            }
        };

        Ok(input)
    }

    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        if let Some(input) = self.input_ring.pop_front() {
            return Ok(Some(input));
        }
        self.backend.read_timeout(&mut self.input_buffer, timeout)?;
        self.drain_input_into_ring()?;
        Ok(self.input_ring.pop_front())
    }

    fn drain_input_into_ring(&mut self) -> Result<()> {
        Self::populate_input_ring(&mut self.input_ring,
                                  &self.ti_cache.escape_sequence_prefix_tree,
                                  &self.input_buffer)?;
        self.input_buffer.clear();
        Ok(())
    }

    fn chip_char(s: &str) -> Option<(char, &str)> {
        let mut chars = s.chars();
        if let Some(first) = chars.next() {
            Some((first, chars.as_str()))
        } else {
            None
        }
    }

    fn populate_input_ring(input_ring: &mut VecDeque<Input>,
                           prefix_tree: &BytePrefixTree<Input>,
                           slice: &[u8]) -> Result<()> {
        let rest = match prefix_tree.get_longest(slice) {
            None => {
                // slice does not begin with an escape sequence - chip off the start and try again
                let s = ::std::str::from_utf8(&slice)?;
                if let Some((ch, rest)) = Self::chip_char(s) {
                    input_ring.push_back(Input::Char(ch));
                    Some(rest.as_bytes())
                } else {
                    None
                }
            }
            Some(Found::Exact(input)) => {
                input_ring.push_back(*input);
                None
            }
            Some(Found::WithRemaining(input, remaining)) => {
                input_ring.push_back(*input);
                Some(remaining)
            }
        };

        if let Some(rest) = rest {
            Self::populate_input_ring(input_ring, prefix_tree, rest)?;
        }

        Ok(())
    }

    pub fn add_char_to_buffer(&mut self, ch: char) {
        self.output_buffer.push(ch);
    }

    pub fn add_str_to_buffer(&mut self, s: &str) {
        self.output_buffer.push_str(s);
    }

    pub fn flush_buffer(&mut self) -> Result<()> {
        self.backend.send(&self.output_buffer)?;
        self.output_buffer.clear();
        Ok(())
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output_buffer.push_str(::std::str::from_utf8(buf).unwrap());
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
