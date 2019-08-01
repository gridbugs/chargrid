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

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum KeyboardInput {
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

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MouseInput {
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

/// An input event
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Input {
    Keyboard(KeyboardInput),
    Mouse(MouseInput),
}

impl Input {
    pub fn is_keyboard(&self) -> bool {
        match self {
            &Input::Keyboard(_) => true,
            &Input::Mouse(_) => false,
        }
    }
}

pub mod keys {
    use super::KeyboardInput;

    pub const ESCAPE: KeyboardInput = KeyboardInput::Char('\u{1b}');
    pub const ETX: KeyboardInput = KeyboardInput::Char('\u{3}');
    pub const BACKSPACE: KeyboardInput = KeyboardInput::Char('\u{8}');
    pub const TAB: KeyboardInput = KeyboardInput::Char('\u{9}');
    pub const RETURN: KeyboardInput = KeyboardInput::Char('\u{d}');
}
