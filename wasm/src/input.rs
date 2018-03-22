use std::slice;
use prototty::{inputs, Input};

const MOD_SHIFT: u8 = (1 << 0);

macro_rules! convert_char_shift {
    ($lower:expr, $upper:expr, $shift:expr) => {
        Some(Input::Char(if $shift {
            $upper
        } else {
            $lower
        }))
    }
}

pub fn from_js_event(event: u64) -> Option<Input> {
    let event_key_code = event as u8;
    let event_key_modifiers = (event >> 8) as u8;

    let shift = event_key_modifiers & MOD_SHIFT != 0;

    match event_key_code {
        8 => Some(inputs::BACKSPACE),
        9 => Some(inputs::TAB),

        13 => Some(inputs::RETURN),

        27 => Some(inputs::ESCAPE),

        32 => Some(Input::Char(' ')),
        33 => Some(Input::PageUp),
        34 => Some(Input::PageDown),
        35 => Some(Input::End),
        36 => Some(Input::Home),
        37 => Some(Input::Left),
        38 => Some(Input::Up),
        39 => Some(Input::Right),
        40 => Some(Input::Down),

        46 => Some(Input::Delete),

        48 => convert_char_shift!('0', ')', shift),
        49 => convert_char_shift!('1', '!', shift),
        50 => convert_char_shift!('2', '@', shift),
        51 => convert_char_shift!('3', '#', shift),
        52 => convert_char_shift!('4', '$', shift),
        53 => convert_char_shift!('5', '%', shift),
        54 => convert_char_shift!('6', '^', shift),
        55 => convert_char_shift!('7', '&', shift),
        56 => convert_char_shift!('8', '*', shift),
        57 => convert_char_shift!('9', '(', shift),

        65 => convert_char_shift!('a', 'A', shift),
        66 => convert_char_shift!('b', 'B', shift),
        67 => convert_char_shift!('c', 'C', shift),
        68 => convert_char_shift!('d', 'D', shift),
        69 => convert_char_shift!('e', 'E', shift),
        70 => convert_char_shift!('f', 'F', shift),
        71 => convert_char_shift!('g', 'G', shift),
        72 => convert_char_shift!('h', 'H', shift),
        73 => convert_char_shift!('i', 'I', shift),
        74 => convert_char_shift!('j', 'J', shift),
        75 => convert_char_shift!('k', 'K', shift),
        76 => convert_char_shift!('l', 'L', shift),
        77 => convert_char_shift!('m', 'M', shift),
        78 => convert_char_shift!('n', 'N', shift),
        79 => convert_char_shift!('o', 'O', shift),
        80 => convert_char_shift!('p', 'P', shift),
        81 => convert_char_shift!('q', 'Q', shift),
        82 => convert_char_shift!('r', 'R', shift),
        83 => convert_char_shift!('s', 'S', shift),
        84 => convert_char_shift!('t', 'T', shift),
        85 => convert_char_shift!('u', 'U', shift),
        86 => convert_char_shift!('v', 'V', shift),
        87 => convert_char_shift!('w', 'W', shift),
        88 => convert_char_shift!('x', 'X', shift),
        89 => convert_char_shift!('y', 'Y', shift),
        90 => convert_char_shift!('z', 'Z', shift),

        96 => Some(Input::Char('0')),
        97 => Some(Input::Char('1')),
        98 => Some(Input::Char('2')),
        99 => Some(Input::Char('3')),
        100 => Some(Input::Char('4')),
        101 => Some(Input::Char('5')),
        102 => Some(Input::Char('6')),
        103 => Some(Input::Char('7')),
        104 => Some(Input::Char('8')),
        105 => Some(Input::Char('9')),

        106 => Some(Input::Char('*')),
        107 => Some(Input::Char('+')),
        109 => Some(Input::Char('-')),
        110 => Some(Input::Char('.')),
        111 => Some(Input::Char('/')),

        112 => Some(Input::Function(1)),
        113 => Some(Input::Function(2)),
        114 => Some(Input::Function(3)),
        115 => Some(Input::Function(4)),
        116 => Some(Input::Function(5)),
        117 => Some(Input::Function(6)),
        118 => Some(Input::Function(7)),
        119 => Some(Input::Function(8)),
        120 => Some(Input::Function(9)),
        121 => Some(Input::Function(10)),
        122 => Some(Input::Function(11)),
        123 => Some(Input::Function(12)),

        186 => convert_char_shift!(';', ':', shift),
        187 => convert_char_shift!('=', '+', shift),
        188 => convert_char_shift!(',', '<', shift),
        189 => convert_char_shift!('-', '_', shift),
        190 => convert_char_shift!('.', '>', shift),
        191 => convert_char_shift!('/', '?', shift),
        192 => convert_char_shift!('`', '~', shift),

        219 => convert_char_shift!('[', '{', shift),
        220 => convert_char_shift!('\\', '|', shift),
        221 => convert_char_shift!(']', '}', shift),
        222 => convert_char_shift!('\'', '"', shift),

        _ => None,
    }
}

pub unsafe fn js_event_input_iter<'a>(
    inputs: *const u64,
    num_inputs: usize,
) -> InputIter<'a> {
    InputIter::new(inputs, num_inputs)
}

pub struct InputIter<'a> {
    inputs: slice::Iter<'a, u64>,
}

impl<'a> InputIter<'a> {
    pub unsafe fn new(inputs: *const u64, num_inputs: usize) -> Self {
        Self {
            inputs: slice::from_raw_parts(inputs, num_inputs).iter(),
        }
    }
}

impl<'a> Iterator for InputIter<'a> {
    type Item = Input;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(input) = self.inputs.next() {
                if let Some(input) = from_js_event(*input) {
                    return Some(input);
                }
            } else {
                return None;
            }
        }
    }
}
