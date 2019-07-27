use super::byte_prefix_tree::{BytePrefixTree, Found};
use super::low_level::LowLevel;
use super::term_info_cache::{MousePrefix, TermInfoCache, TerminalInput};
use error::Result;
use prototty_input::*;
use prototty_render::*;
use rgb24::Rgb24;
use std::collections::{vec_deque, VecDeque};
use std::time::Duration;
use term::terminfo::parm::{self, Param};

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

pub mod encode_colour {
    use crate::terminal::ansi_colour_codes::{nearest_ansi_code, nearest_mean_greyscale_code, nearest_palette_code};
    use crate::terminal::term_info_cache::TermInfoCache;
    use rgb24::Rgb24;

    pub trait Trait {
        fn encode_foreground(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache);
        fn encode_background(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache);
    }

    #[derive(Clone, Copy)]
    pub struct FromTermInfoRgb;
    impl Trait for FromTermInfoRgb {
        fn encode_foreground(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.fg_colour(nearest_palette_code(rgb24)));
        }
        fn encode_background(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.bg_colour(nearest_palette_code(rgb24)));
        }
    }

    #[derive(Clone, Copy)]
    pub struct FromTermInfoGreyscale;
    impl Trait for FromTermInfoGreyscale {
        fn encode_foreground(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.fg_colour(nearest_mean_greyscale_code(rgb24)));
        }
        fn encode_background(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.bg_colour(nearest_mean_greyscale_code(rgb24)));
        }
    }

    #[derive(Clone, Copy)]
    pub struct FromTermInfoAnsi16Colour;
    impl Trait for FromTermInfoAnsi16Colour {
        fn encode_foreground(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.fg_colour(nearest_ansi_code(rgb24)));
        }
        fn encode_background(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {
            buffer.push_str(term_info_cache.bg_colour(nearest_ansi_code(rgb24)));
        }
    }

    #[derive(Clone, Copy)]
    pub struct TrueColour;
    impl Trait for TrueColour {
        fn encode_foreground(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {}
        fn encode_background(buffer: &mut String, rgb24: Rgb24, term_info_cache: &TermInfoCache) {}
    }
}

pub use self::encode_colour::Trait as EncodeColour;

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
        if let Some(enter_ca) = self.ti_cache.enter_ca.as_ref() {
            self.output_buffer.push_str(enter_ca);
        }
        if let Some(enter_xmit) = self.ti_cache.enter_xmit.as_ref() {
            self.output_buffer.push_str(enter_xmit);
        }
        self.output_buffer.push_str(&self.ti_cache.hide_cursor);
        self.output_buffer.push_str(&self.ti_cache.clear);
        self.output_buffer.push_str(&self.ti_cache.enable_mouse_reporting);
        self.flush_buffer().map_err(Into::into)
    }

    fn teardown(&mut self) -> Result<()> {
        self.output_buffer.push_str(&self.ti_cache.disable_mouse_reporting);
        if let Some(exit_ca) = self.ti_cache.exit_ca.as_ref() {
            self.output_buffer.push_str(exit_ca);
        }
        if let Some(exit_xmit) = self.ti_cache.exit_xmit.as_ref() {
            self.output_buffer.push_str(exit_xmit);
        }
        self.output_buffer.push_str(&self.ti_cache.show_cursor);
        self.output_buffer.push_str(&self.ti_cache.reset);
        self.flush_buffer().map_err(Into::into)
    }

    pub fn size(&self) -> Result<Size> {
        self.low_level.size()
    }

    pub fn set_cursor(&mut self, coord: Coord) -> Result<()> {
        let params = &[Param::Number(coord.y), Param::Number(coord.x)];
        let command = parm::expand(self.ti_cache.set_cursor.as_bytes(), params, &mut self.ti_cache.vars)?;
        let command_slice = ::std::str::from_utf8(&command)?;
        self.output_buffer.push_str(command_slice);
        Ok(())
    }

    pub fn set_foreground_colour<E>(&mut self, rgb24: Rgb24)
    where
        E: EncodeColour,
    {
        E::encode_foreground(&mut self.output_buffer, rgb24, &self.ti_cache);
    }

    pub fn set_background_colour<E>(&mut self, rgb24: Rgb24)
    where
        E: EncodeColour,
    {
        E::encode_background(&mut self.output_buffer, rgb24, &self.ti_cache);
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
            if let Some(input) = self.input_ring.pop_front() {
                break input;
            }
            self.low_level.read_waiting(&mut self.input_buffer)?;
            self.drain_input_into_ring()?;
        };
        Ok(input)
    }

    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        if let Some(input) = self.input_ring.pop_front() {
            return Ok(Some(input));
        }
        self.low_level.read_timeout(&mut self.input_buffer, timeout)?;
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
        prefix_tree: &BytePrefixTree<TerminalInput>,
        slice: &[u8],
    ) -> Result<()> {
        let (term_input, rest) = match prefix_tree.get_longest(slice) {
            None => {
                // slice does not begin with an escape sequence - chip off the start and try again
                let s = if let Ok(s) = ::std::str::from_utf8(&slice) {
                    s
                } else {
                    return Ok(());
                };
                if let Some((ch, rest)) = Self::chip_char(s) {
                    (Some(TerminalInput::Char(ch)), Some(rest.as_bytes()))
                } else {
                    (None, None)
                }
            }
            Some(Found::Exact(input)) => (Some(*input), None),
            Some(Found::WithRemaining(input, remaining)) => (Some(*input), Some(remaining)),
        };
        let rest = if let Some(input) = term_input {
            let (input, rest) = match input {
                TerminalInput::Char(ch) => (Some(Input::Char(ch)), rest),
                TerminalInput::Literal(input) => (Some(input), rest),
                TerminalInput::MousePrefix(prefix) => {
                    if let Some(rest) = rest {
                        if rest.len() >= 2 {
                            const COORD_OFFSET: i32 = 33;
                            let x = rest[0] as i32 - COORD_OFFSET;
                            let y = rest[1] as i32 - COORD_OFFSET;
                            let coord = Coord::new(x, y);
                            let input = match prefix {
                                MousePrefix::Move(button) => Input::MouseMove { button, coord },
                                MousePrefix::Press(button) => Input::MousePress { button, coord },
                                MousePrefix::Release => Input::MouseRelease {
                                    button: Err(NotSupported),
                                    coord,
                                },
                                MousePrefix::Scroll(direction) => Input::MouseScroll { direction, coord },
                            };
                            (Some(input), Some(&rest[2..]))
                        } else {
                            (None, Some(rest))
                        }
                    } else {
                        (None, None)
                    }
                }
            };
            if let Some(input) = input {
                input_ring.push_back(input);
            }
            rest
        } else {
            rest
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
        self.teardown().expect("Failed to reset terminal to original settings");
    }
}
