use crate::Dimensions;
use chargrid_input::{
    keys, Coord, Input, KeyboardInput, MouseButton as ChargridMouseButton, MouseButton, MouseInput,
    ScrollDirection,
};
use winit::{
    dpi::{LogicalPosition, PhysicalSize},
    event::{ElementState, MouseButton as GlutinMouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

pub enum Event {
    Input(Input),
    Resize(PhysicalSize<u32>),
}

macro_rules! convert_char_shift {
    ($lower:expr, $upper:expr, $shift:expr) => {
        KeyboardInput::Char(if $shift { $upper } else { $lower })
    };
}

#[allow(clippy::cognitive_complexity)]
fn convert_keycode_keyboard_input(code: KeyCode, shift: bool) -> Option<KeyboardInput> {
    let keyboard_input = match code {
        KeyCode::Space => KeyboardInput::Char(' '),
        KeyCode::KeyA => convert_char_shift!('a', 'A', shift),
        KeyCode::KeyB => convert_char_shift!('b', 'B', shift),
        KeyCode::KeyC => convert_char_shift!('c', 'C', shift),
        KeyCode::KeyD => convert_char_shift!('d', 'D', shift),
        KeyCode::KeyE => convert_char_shift!('e', 'E', shift),
        KeyCode::KeyF => convert_char_shift!('f', 'F', shift),
        KeyCode::KeyG => convert_char_shift!('g', 'G', shift),
        KeyCode::KeyH => convert_char_shift!('h', 'H', shift),
        KeyCode::KeyI => convert_char_shift!('i', 'I', shift),
        KeyCode::KeyJ => convert_char_shift!('j', 'J', shift),
        KeyCode::KeyK => convert_char_shift!('k', 'K', shift),
        KeyCode::KeyL => convert_char_shift!('l', 'L', shift),
        KeyCode::KeyM => convert_char_shift!('m', 'M', shift),
        KeyCode::KeyN => convert_char_shift!('n', 'N', shift),
        KeyCode::KeyO => convert_char_shift!('o', 'O', shift),
        KeyCode::KeyP => convert_char_shift!('p', 'P', shift),
        KeyCode::KeyQ => convert_char_shift!('q', 'Q', shift),
        KeyCode::KeyR => convert_char_shift!('r', 'R', shift),
        KeyCode::KeyS => convert_char_shift!('s', 'S', shift),
        KeyCode::KeyT => convert_char_shift!('t', 'T', shift),
        KeyCode::KeyU => convert_char_shift!('u', 'U', shift),
        KeyCode::KeyV => convert_char_shift!('v', 'V', shift),
        KeyCode::KeyW => convert_char_shift!('w', 'W', shift),
        KeyCode::KeyX => convert_char_shift!('x', 'X', shift),
        KeyCode::KeyY => convert_char_shift!('y', 'Y', shift),
        KeyCode::KeyZ => convert_char_shift!('z', 'Z', shift),
        KeyCode::Digit1 => convert_char_shift!('1', '!', shift),
        KeyCode::Digit2 => convert_char_shift!('2', '@', shift),
        KeyCode::Digit3 => convert_char_shift!('3', '#', shift),
        KeyCode::Digit4 => convert_char_shift!('4', '$', shift),
        KeyCode::Digit5 => convert_char_shift!('5', '%', shift),
        KeyCode::Digit6 => convert_char_shift!('6', '^', shift),
        KeyCode::Digit7 => convert_char_shift!('7', '&', shift),
        KeyCode::Digit8 => convert_char_shift!('8', '*', shift),
        KeyCode::Digit9 => convert_char_shift!('9', '(', shift),
        KeyCode::Digit0 => convert_char_shift!('0', ')', shift),
        KeyCode::Numpad1 => KeyboardInput::Char('1'),
        KeyCode::Numpad2 => KeyboardInput::Char('2'),
        KeyCode::Numpad3 => KeyboardInput::Char('3'),
        KeyCode::Numpad4 => KeyboardInput::Char('4'),
        KeyCode::Numpad5 => KeyboardInput::Char('5'),
        KeyCode::Numpad6 => KeyboardInput::Char('6'),
        KeyCode::Numpad7 => KeyboardInput::Char('7'),
        KeyCode::Numpad8 => KeyboardInput::Char('8'),
        KeyCode::Numpad9 => KeyboardInput::Char('9'),
        KeyCode::Numpad0 => KeyboardInput::Char('0'),
        KeyCode::ArrowLeft => KeyboardInput::Left,
        KeyCode::ArrowRight => KeyboardInput::Right,
        KeyCode::ArrowUp => KeyboardInput::Up,
        KeyCode::ArrowDown => KeyboardInput::Down,
        KeyCode::Escape => keys::ESCAPE,
        KeyCode::Enter => keys::RETURN,
        KeyCode::Minus => KeyboardInput::Char('-'),
        KeyCode::Equal => convert_char_shift!('=', '+', shift),
        KeyCode::Backslash => convert_char_shift!('\\', '|', shift),
        KeyCode::Backquote => convert_char_shift!('`', '~', shift),
        KeyCode::Quote => convert_char_shift!('\'', '"', shift),
        KeyCode::BracketLeft => convert_char_shift!('[', '{', shift),
        KeyCode::BracketRight => convert_char_shift!(']', '}', shift),
        KeyCode::PageUp => KeyboardInput::PageUp,
        KeyCode::PageDown => KeyboardInput::PageDown,
        KeyCode::Home => KeyboardInput::Home,
        KeyCode::End => KeyboardInput::End,
        KeyCode::F1 => KeyboardInput::Function(1),
        KeyCode::F2 => KeyboardInput::Function(2),
        KeyCode::F3 => KeyboardInput::Function(3),
        KeyCode::F4 => KeyboardInput::Function(4),
        KeyCode::F5 => KeyboardInput::Function(5),
        KeyCode::F6 => KeyboardInput::Function(6),
        KeyCode::F7 => KeyboardInput::Function(7),
        KeyCode::F8 => KeyboardInput::Function(8),
        KeyCode::F9 => KeyboardInput::Function(9),
        KeyCode::F10 => KeyboardInput::Function(10),
        KeyCode::F11 => KeyboardInput::Function(11),
        KeyCode::F12 => KeyboardInput::Function(12),
        KeyCode::F13 => KeyboardInput::Function(13),
        KeyCode::F14 => KeyboardInput::Function(14),
        KeyCode::F15 => KeyboardInput::Function(15),
        KeyCode::F16 => KeyboardInput::Function(16),
        KeyCode::F17 => KeyboardInput::Function(17),
        KeyCode::F18 => KeyboardInput::Function(18),
        KeyCode::F19 => KeyboardInput::Function(19),
        KeyCode::F20 => KeyboardInput::Function(20),
        KeyCode::F21 => KeyboardInput::Function(21),
        KeyCode::F22 => KeyboardInput::Function(22),
        KeyCode::F23 => KeyboardInput::Function(23),
        KeyCode::F24 => KeyboardInput::Function(24),
        KeyCode::Backspace => keys::BACKSPACE,
        KeyCode::Delete => KeyboardInput::Delete,
        KeyCode::Comma => convert_char_shift!(',', '<', shift),
        KeyCode::Period => convert_char_shift!('.', '>', shift),
        KeyCode::Slash => convert_char_shift!('/', '?', shift),
        _ => return None,
    };
    Some(keyboard_input)
}

pub fn convert_event(
    event: WindowEvent,
    cell_dimensions: Dimensions<f64>,
    top_left_position: Dimensions<f64>,
    last_mouse_coord: &mut Coord,
    last_mouse_button: &mut Option<MouseButton>,
    scale_factor: &mut f64,
    modifier_state: ModifiersState,
) -> Option<Event> {
    match event {
        WindowEvent::CloseRequested => Some(Event::Input(Input::Keyboard(keys::ETX))),
        WindowEvent::Resized(physical_size) => Some(Event::Resize(physical_size)),
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state == ElementState::Pressed {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if let Some(keyboard_input) =
                        convert_keycode_keyboard_input(key_code, modifier_state.shift_key())
                    {
                        return Some(Event::Input(Input::Keyboard(keyboard_input)));
                    }
                }
            }
            None
        }
        WindowEvent::CursorMoved {
            position: physical_position,
            ..
        } => {
            let LogicalPosition { x, y }: LogicalPosition<f64> =
                physical_position.to_logical(*scale_factor);
            let x = ((x - top_left_position.width) / cell_dimensions.width) as i32;
            let y = ((y - top_left_position.height) / cell_dimensions.height) as i32;
            let coord = Coord::new(x, y);
            *last_mouse_coord = coord;
            Some(Event::Input(Input::Mouse(MouseInput::MouseMove {
                coord,
                button: *last_mouse_button,
            })))
        }
        WindowEvent::MouseInput { state, button, .. } => {
            let button = match button {
                GlutinMouseButton::Left => ChargridMouseButton::Left,
                GlutinMouseButton::Middle => ChargridMouseButton::Middle,
                GlutinMouseButton::Right => ChargridMouseButton::Right,
                _ => return None,
            };
            let input = match state {
                ElementState::Pressed => {
                    *last_mouse_button = Some(button);
                    Input::Mouse(MouseInput::MousePress {
                        coord: *last_mouse_coord,
                        button,
                    })
                }
                ElementState::Released => {
                    *last_mouse_button = None;
                    Input::Mouse(MouseInput::MouseRelease {
                        coord: *last_mouse_coord,
                        button: Ok(button),
                    })
                }
            };
            Some(Event::Input(input))
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (x, y),
                MouseScrollDelta::PixelDelta(physical_position) => {
                    let LogicalPosition { x, y } =
                        physical_position.to_logical::<f64>(*scale_factor);
                    (x as f32, y as f32)
                }
            };
            let direction = if y > 0. {
                ScrollDirection::Up
            } else if y < 0. {
                ScrollDirection::Down
            } else if x > 0. {
                ScrollDirection::Right
            } else if x < 0. {
                ScrollDirection::Left
            } else {
                return None;
            };
            Some(Event::Input(Input::Mouse(MouseInput::MouseScroll {
                direction,
                coord: *last_mouse_coord,
            })))
        }
        _ => None,
    }
}
