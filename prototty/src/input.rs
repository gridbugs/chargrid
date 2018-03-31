use grid_2d::Coord;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

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
    MouseMove(Coord),
    MousePress(Coord),
    MouseRelease(Coord),
    MouseDrag(Coord),
    MouseScroll {
        direction: ScrollDirection,
        coord: Coord,
    },
}

pub mod inputs {
    use super::Input;

    pub const ESCAPE: Input = Input::Char('\u{1b}');
    pub const ETX: Input = Input::Char('\u{3}');
    pub const BACKSPACE: Input = Input::Char('\u{8}');
    pub const TAB: Input = Input::Char('\u{9}');
    pub const RETURN: Input = Input::Char('\u{d}');
}
