#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;
extern crate coord_2d;
pub use coord_2d::Coord;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NotSupported;

/// An input event
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    MouseMove {
        button: Option<MouseButton>,
        coord: Coord,
    },
    MousePress {
        button: MouseButton,
        coord: Coord,
    },
    MouseRelease {
        // some platforms (e.g. ansi terminal) don't report the button that was released
        button: Result<MouseButton, NotSupported>,
        coord: Coord,
    },
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
