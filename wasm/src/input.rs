use prototty::{inputs, Input};

pub fn from_js_event(event_which: u8, event_key_code: u8) -> Option<Input> {
    if event_which != 0 {
        return Some(Input::Char(event_which as char));
    }
    match event_key_code {
        27 => Some(inputs::ESCAPE),
        33 => Some(Input::PageUp),
        34 => Some(Input::PageDown),
        35 => Some(Input::End),
        36 => Some(Input::Home),
        37 => Some(Input::Left),
        38 => Some(Input::Up),
        39 => Some(Input::Right),
        40 => Some(Input::Down),
        46 => Some(Input::Delete),
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
        _ => None,
    }
}
