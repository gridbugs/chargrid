use std::collections::{vec_deque, VecDeque};
use std::time::Duration;
use term::terminfo::parm::{self, Param};
use ansi_colour::Colour;
use super::low_level::LowLevel;
use super::term_info_cache::TermInfoCache;
use super::byte_prefix_tree::{BytePrefixTree, Found};
use error::Result;
use prototty::*;

const OUTPUT_BUFFER_INITIAL_CAPACITY: usize = 32 * 1024;
const INPUT_BUFFER_INITIAL_CAPACITY: usize = 32;
const INPUT_RING_INITIAL_CAPACITY: usize = 32;

pub type DrainInput<'a> = vec_deque::Drain<'a, Input>;

pub struct AnsiTerminal {
    low_level: LowLevel,
    output_buffer: String,
    input_buffer: Vec<u8>,
    ti_cache: TermInfoCache,
    input_ring: VecDeque<Input>,
}

impl AnsiTerminal {
    pub fn new() -> Result<Self> {
        let low_level = LowLevel::new()?;
        let output_buffer = String::with_capacity(OUTPUT_BUFFER_INITIAL_CAPACITY);
        let input_buffer = Vec::with_capacity(INPUT_BUFFER_INITIAL_CAPACITY);
        let ti_cache = TermInfoCache::new()?;
        let input_ring = VecDeque::with_capacity(INPUT_RING_INITIAL_CAPACITY);

        let mut terminal = Self {
            low_level,
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

        self.flush_buffer().map_err(Into::into)
    }

    fn teardown(&mut self) -> Result<()> {
        self.output_buffer.push_str(&self.ti_cache.exit_ca);
        self.output_buffer.push_str(&self.ti_cache.exit_xmit);
        self.output_buffer.push_str(&self.ti_cache.show_cursor);
        self.output_buffer.push_str(&self.ti_cache.reset);

        self.flush_buffer().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Size> {
        self.low_level.size()
    }

    pub fn set_cursor(&mut self, coord: Coord) -> Result<()> {
        let params = &[Param::Number(coord.y), Param::Number(coord.x)];
        let command = parm::expand(
            self.ti_cache.set_cursor.as_bytes(),
            params,
            &mut self.ti_cache.vars,
        )?;
        let command_slice = ::std::str::from_utf8(&command)?;
        self.output_buffer.push_str(command_slice);
        Ok(())
    }

    pub fn set_foreground_colour(&mut self, colour: Colour) {
        self.output_buffer.push_str(self.ti_cache.fg_colour(colour));
    }

    pub fn set_background_colour(&mut self, colour: Colour) {
        self.output_buffer.push_str(self.ti_cache.bg_colour(colour));
    }

    pub fn set_bold(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.bold);
    }

    pub fn set_underline(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.underline);
    }

    pub fn clear_underline(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.no_underline);
    }

    pub fn reset(&mut self) {
        self.output_buffer.push_str(&self.ti_cache.reset);
    }

    pub fn drain_input(&mut self) -> Result<DrainInput> {
        self.low_level.read_polling(&mut self.input_buffer)?;
        self.drain_input_into_ring()?;
        Ok(self.input_ring.drain(..))
    }

    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        if let Some(input) = self.input_ring.pop_front() {
            return Ok(Some(input));
        }
        self.low_level.read_polling(&mut self.input_buffer)?;
        self.drain_input_into_ring()?;
        Ok(self.input_ring.pop_front())
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        let input = loop {
            // This loop is for good measure. If for some reason
            // read_waiting returns without a new input becoming
            // available, this will ensure we don't return without
            // an input.
            self.low_level.read_waiting(&mut self.input_buffer)?;
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
        self.low_level
            .read_timeout(&mut self.input_buffer, timeout)?;
        self.drain_input_into_ring()?;
        Ok(self.input_ring.pop_front())
    }

    fn drain_input_into_ring(&mut self) -> Result<()> {
        Self::populate_input_ring(
            &mut self.input_ring,
            &self.ti_cache.escape_sequence_prefix_tree,
            &self.input_buffer,
        )?;
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

    fn populate_input_ring(
        input_ring: &mut VecDeque<Input>,
        prefix_tree: &BytePrefixTree<Input>,
        slice: &[u8],
    ) -> Result<()> {
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

    pub fn flush_buffer(&mut self) -> Result<()> {
        self.low_level.send(&self.output_buffer)?;
        self.output_buffer.clear();
        Ok(())
    }
}

impl Drop for AnsiTerminal {
    fn drop(&mut self) {
        self.teardown()
            .expect("Failed to reset terminal to original settings");
    }
}
