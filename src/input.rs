/// An input event
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Input {
    Char(char),
    Function(u8),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
}

pub mod inputs {
    use super::Input;

    pub const ESCAPE: Input = Input::Char('\u{1b}');
    pub const ETX: Input = Input::Char('\u{3}');
    pub const RETURN: Input = Input::Char('\u{d}');
}
