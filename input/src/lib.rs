pub use coord_2d::Coord;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

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

#[cfg(feature = "serialize")]
mod key_names {
    pub const UP: &str = "up";
    pub const DOWN: &str = "down";
    pub const LEFT: &str = "left";
    pub const RIGHT: &str = "right";
    pub const HOME: &str = "home";
    pub const END: &str = "end";
    pub const PAGE_UP: &str = "page-up";
    pub const PAGE_DOWN: &str = "page-down";
    pub const DELETE: &str = "delete";
}

#[cfg(feature = "serialize")]
impl KeyboardInput {
    fn try_from_str(s: &str) -> Option<Self> {
        if s.chars().count() == 1 {
            let c = s.chars().next().unwrap();
            return Some(KeyboardInput::Char(c));
        }
        if s.starts_with('f') || s.starts_with('F') {
            let (_, maybe_number_str) = s.split_at(1);
            if let Ok(number) = maybe_number_str.parse::<u8>() {
                return Some(KeyboardInput::Function(number));
            }
        }
        use key_names::*;
        match s {
            UP => Some(KeyboardInput::Up),
            DOWN => Some(KeyboardInput::Down),
            LEFT => Some(KeyboardInput::Left),
            RIGHT => Some(KeyboardInput::Right),
            HOME => Some(KeyboardInput::Home),
            END => Some(KeyboardInput::End),
            PAGE_UP => Some(KeyboardInput::PageUp),
            PAGE_DOWN => Some(KeyboardInput::PageDown),
            DELETE => Some(KeyboardInput::Delete),
            _ => None,
        }
    }
}

#[cfg(feature = "serialize")]
impl serde::Serialize for KeyboardInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use key_names::*;
        use KeyboardInput::*;
        match self {
            Char(c) => serializer.serialize_char(*c),
            Function(n) => serializer.serialize_str(&format!("f{}", n)),
            Up => serializer.serialize_str(UP),
            Down => serializer.serialize_str(DOWN),
            Left => serializer.serialize_str(LEFT),
            Right => serializer.serialize_str(RIGHT),
            Home => serializer.serialize_str(HOME),
            End => serializer.serialize_str(END),
            PageUp => serializer.serialize_str(PAGE_UP),
            PageDown => serializer.serialize_str(PAGE_DOWN),
            Delete => serializer.serialize_str(DELETE),
        }
    }
}

#[cfg(feature = "serialize")]
impl<'de> serde::Deserialize<'de> for KeyboardInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyboardInput;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a keyboard input description")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                KeyboardInput::try_from_str(s)
                    .ok_or_else(|| E::custom(format!("couldn't parse {}", s)))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
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

impl MouseInput {
    pub fn coord(&self) -> Coord {
        match self {
            Self::MouseMove { coord, .. }
            | Self::MousePress { coord, .. }
            | Self::MouseRelease { coord, .. }
            | Self::MouseScroll { coord, .. } => *coord,
        }
    }

    fn coord_mut(&mut self) -> &mut Coord {
        match self {
            Self::MouseMove { coord, .. }
            | Self::MousePress { coord, .. }
            | Self::MouseRelease { coord, .. }
            | Self::MouseScroll { coord, .. } => coord,
        }
    }

    pub fn relative_to_coord(&self, coord: Coord) -> Self {
        let mut ret = *self;
        *ret.coord_mut() -= coord;
        ret
    }
}

#[cfg(feature = "gamepad")]
mod gamepad {
    #[cfg(feature = "serialize")]
    use serde::{Deserialize, Serialize};

    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub enum GamepadButton {
        DPadUp,
        DPadRight,
        DPadDown,
        DPadLeft,
        North,
        East,
        South,
        West,
        Start,
        Select,
        LeftBumper,
        RightBumper,
    }

    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct GamepadInput {
        pub button: GamepadButton,
        pub id: u64,
    }
}

#[cfg(feature = "gamepad")]
pub use gamepad::{GamepadButton, GamepadInput};

/// Opinionated policy for interpreting input for convenience
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InputPolicy {
    Up,
    Down,
    Left,
    Right,
    Select,
}

/// An input event
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Input {
    Keyboard(KeyboardInput),
    Mouse(MouseInput),
    #[cfg(feature = "gamepad")]
    Gamepad(GamepadInput),
}

impl Input {
    pub fn is_keyboard(&self) -> bool {
        match self {
            Input::Keyboard(_) => true,
            Input::Mouse(_) => false,
            #[cfg(feature = "gamepad")]
            Input::Gamepad(_) => false,
        }
    }

    pub fn is_mouse(&self) -> bool {
        match self {
            Input::Keyboard(_) => false,
            Input::Mouse(_) => true,
            #[cfg(feature = "gamepad")]
            Input::Gamepad(_) => false,
        }
    }

    #[cfg(feature = "gamepad")]
    pub fn is_gamepad(&self) -> bool {
        match self {
            Input::Keyboard(_) => false,
            Input::Mouse(_) => false,
            Input::Gamepad(_) => true,
        }
    }

    pub fn keyboard(self) -> Option<KeyboardInput> {
        match self {
            Input::Keyboard(keyboard_input) => Some(keyboard_input),
            Input::Mouse(_) => None,
            #[cfg(feature = "gamepad")]
            Input::Gamepad(_) => None,
        }
    }

    pub fn mouse(self) -> Option<MouseInput> {
        match self {
            Input::Keyboard(_) => None,
            Input::Mouse(mouse_input) => Some(mouse_input),
            #[cfg(feature = "gamepad")]
            Input::Gamepad(_) => None,
        }
    }

    #[cfg(feature = "gamepad")]
    pub fn gamepad(self) -> Option<GamepadInput> {
        match self {
            Input::Keyboard(_) | Input::Mouse(_) => None,
            Input::Gamepad(gamepad_input) => Some(gamepad_input),
        }
    }

    pub fn policy(self) -> Option<InputPolicy> {
        match self {
            Input::Keyboard(KeyboardInput::Left) => Some(InputPolicy::Left),
            Input::Keyboard(KeyboardInput::Right) => Some(InputPolicy::Right),
            Input::Keyboard(keys::RETURN) => Some(InputPolicy::Select),
            Input::Mouse(MouseInput::MousePress { .. }) => Some(InputPolicy::Select),
            Input::Mouse(MouseInput::MouseScroll { direction, .. }) => Some(match direction {
                ScrollDirection::Up => InputPolicy::Up,
                ScrollDirection::Down => InputPolicy::Down,
                ScrollDirection::Left => InputPolicy::Left,
                ScrollDirection::Right => InputPolicy::Right,
            }),
            #[cfg(feature = "gamepad")]
            Input::Gamepad(GamepadInput { button, .. }) => match button {
                GamepadButton::DPadLeft => Some(InputPolicy::Left),
                GamepadButton::DPadRight => Some(InputPolicy::Right),
                GamepadButton::Start | GamepadButton::South => Some(InputPolicy::Select),
                _ => None,
            },
            _ => None,
        }
    }
}

pub mod keys {
    use super::KeyboardInput;

    pub const ESCAPE: KeyboardInput = KeyboardInput::Char('\u{1b}');
    pub const ETX: KeyboardInput = KeyboardInput::Char('\u{3}');
    pub const BACKSPACE: KeyboardInput = KeyboardInput::Char('\u{7f}');
    pub const TAB: KeyboardInput = KeyboardInput::Char('\u{9}');
    pub const RETURN: KeyboardInput = KeyboardInput::Char('\u{d}');
}

#[cfg(feature = "serialize")]
#[cfg(test)]
mod serde_test {
    #[test]
    fn reversable() {
        use super::KeyboardInput;
        fn t(input: KeyboardInput) {
            let s = serde_json::to_string(&input).unwrap();
            assert_eq!(serde_json::from_str::<KeyboardInput>(&s).unwrap(), input);
        }
        t(KeyboardInput::Up);
        t(KeyboardInput::Down);
        t(KeyboardInput::Left);
        t(KeyboardInput::Right);
        t(KeyboardInput::Home);
        t(KeyboardInput::End);
        t(KeyboardInput::PageUp);
        t(KeyboardInput::PageDown);
        t(KeyboardInput::Delete);
        t(KeyboardInput::Function(42));
        t(KeyboardInput::Char('a'));
        t(KeyboardInput::Char('☃'));
    }

    #[test]
    fn example() {
        use super::KeyboardInput;
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert(KeyboardInput::Up, "UP");
        map.insert(KeyboardInput::Down, "DOWN");
        map.insert(KeyboardInput::Function(42), "F42");
        map.insert(KeyboardInput::Char('a'), "A");
        map.insert(KeyboardInput::Char('☃'), "SNOWMAN");
        let pretty_json_string = serde_json::to_string_pretty(&map).unwrap();
        assert_eq!(
            pretty_json_string,
            "{
  \"a\": \"A\",
  \"☃\": \"SNOWMAN\",
  \"f42\": \"F42\",
  \"up\": \"UP\",
  \"down\": \"DOWN\"
}",
        );
    }
}
