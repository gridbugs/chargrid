use crate::Dimensions;
use chargrid_input::{
    keys, Input, KeyboardInput, MouseButton as ChargridMouseButton, MouseButton, MouseInput, ScrollDirection,
};
use chargrid_render::Coord;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    ElementState, ModifiersState, MouseButton as GlutinMouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

pub enum Event {
    Input(Input),
    Resize(LogicalSize<f64>),
}

macro_rules! convert_char_shift {
    ($lower:expr, $upper:expr, $shift:expr) => {
        KeyboardInput::Char(if $shift { $upper } else { $lower })
    };
}

#[allow(clippy::cognitive_complexity)]
fn convert_keycode_keyboard_input(code: VirtualKeyCode, shift: bool) -> Option<KeyboardInput> {
    let keyboard_input = match code {
        VirtualKeyCode::Space => KeyboardInput::Char(' '),
        VirtualKeyCode::A => convert_char_shift!('a', 'A', shift),
        VirtualKeyCode::B => convert_char_shift!('b', 'B', shift),
        VirtualKeyCode::C => convert_char_shift!('c', 'C', shift),
        VirtualKeyCode::D => convert_char_shift!('d', 'D', shift),
        VirtualKeyCode::E => convert_char_shift!('e', 'E', shift),
        VirtualKeyCode::F => convert_char_shift!('f', 'F', shift),
        VirtualKeyCode::G => convert_char_shift!('g', 'G', shift),
        VirtualKeyCode::H => convert_char_shift!('h', 'H', shift),
        VirtualKeyCode::I => convert_char_shift!('i', 'I', shift),
        VirtualKeyCode::J => convert_char_shift!('j', 'J', shift),
        VirtualKeyCode::K => convert_char_shift!('k', 'K', shift),
        VirtualKeyCode::L => convert_char_shift!('l', 'L', shift),
        VirtualKeyCode::M => convert_char_shift!('m', 'M', shift),
        VirtualKeyCode::N => convert_char_shift!('n', 'N', shift),
        VirtualKeyCode::O => convert_char_shift!('o', 'O', shift),
        VirtualKeyCode::P => convert_char_shift!('p', 'P', shift),
        VirtualKeyCode::Q => convert_char_shift!('q', 'Q', shift),
        VirtualKeyCode::R => convert_char_shift!('r', 'R', shift),
        VirtualKeyCode::S => convert_char_shift!('s', 'S', shift),
        VirtualKeyCode::T => convert_char_shift!('t', 'T', shift),
        VirtualKeyCode::U => convert_char_shift!('u', 'U', shift),
        VirtualKeyCode::V => convert_char_shift!('v', 'V', shift),
        VirtualKeyCode::W => convert_char_shift!('w', 'W', shift),
        VirtualKeyCode::X => convert_char_shift!('x', 'X', shift),
        VirtualKeyCode::Y => convert_char_shift!('y', 'Y', shift),
        VirtualKeyCode::Z => convert_char_shift!('z', 'Z', shift),
        VirtualKeyCode::Key1 => convert_char_shift!('1', '!', shift),
        VirtualKeyCode::Key2 => KeyboardInput::Char('2'),
        VirtualKeyCode::Key3 => convert_char_shift!('3', '#', shift),
        VirtualKeyCode::Key4 => convert_char_shift!('4', '$', shift),
        VirtualKeyCode::Key5 => convert_char_shift!('5', '%', shift),
        VirtualKeyCode::Key6 => convert_char_shift!('6', '^', shift),
        VirtualKeyCode::Key7 => convert_char_shift!('7', '&', shift),
        VirtualKeyCode::Key8 => convert_char_shift!('8', '*', shift),
        VirtualKeyCode::Key9 => convert_char_shift!('9', '(', shift),
        VirtualKeyCode::Key0 => convert_char_shift!('0', ')', shift),
        VirtualKeyCode::Numpad1 => KeyboardInput::Char('1'),
        VirtualKeyCode::Numpad2 => KeyboardInput::Char('2'),
        VirtualKeyCode::Numpad3 => KeyboardInput::Char('3'),
        VirtualKeyCode::Numpad4 => KeyboardInput::Char('4'),
        VirtualKeyCode::Numpad5 => KeyboardInput::Char('5'),
        VirtualKeyCode::Numpad6 => KeyboardInput::Char('6'),
        VirtualKeyCode::Numpad7 => KeyboardInput::Char('7'),
        VirtualKeyCode::Numpad8 => KeyboardInput::Char('8'),
        VirtualKeyCode::Numpad9 => KeyboardInput::Char('9'),
        VirtualKeyCode::Numpad0 => KeyboardInput::Char('0'),
        VirtualKeyCode::Period => convert_char_shift!('.', '>', shift),
        VirtualKeyCode::Comma => convert_char_shift!(',', '<', shift),
        VirtualKeyCode::Slash => convert_char_shift!('/', '?', shift),
        VirtualKeyCode::Left => KeyboardInput::Left,
        VirtualKeyCode::Right => KeyboardInput::Right,
        VirtualKeyCode::Up => KeyboardInput::Up,
        VirtualKeyCode::Down => KeyboardInput::Down,
        VirtualKeyCode::Escape => keys::ESCAPE,
        VirtualKeyCode::Return => keys::RETURN,
        VirtualKeyCode::At => KeyboardInput::Char('@'),
        VirtualKeyCode::Add => KeyboardInput::Char('+'),
        VirtualKeyCode::Subtract => KeyboardInput::Char('-'),
        VirtualKeyCode::Equals => convert_char_shift!('=', '+', shift),
        VirtualKeyCode::Backslash => convert_char_shift!('\\', '|', shift),
        VirtualKeyCode::Grave => convert_char_shift!('`', '~', shift),
        VirtualKeyCode::Apostrophe => convert_char_shift!('\'', '"', shift),
        VirtualKeyCode::LBracket => convert_char_shift!('[', '{', shift),
        VirtualKeyCode::RBracket => convert_char_shift!(']', '}', shift),
        VirtualKeyCode::PageUp => KeyboardInput::PageUp,
        VirtualKeyCode::PageDown => KeyboardInput::PageDown,
        VirtualKeyCode::Home => KeyboardInput::Home,
        VirtualKeyCode::End => KeyboardInput::End,
        VirtualKeyCode::F1 => KeyboardInput::Function(1),
        VirtualKeyCode::F2 => KeyboardInput::Function(2),
        VirtualKeyCode::F3 => KeyboardInput::Function(3),
        VirtualKeyCode::F4 => KeyboardInput::Function(4),
        VirtualKeyCode::F5 => KeyboardInput::Function(5),
        VirtualKeyCode::F6 => KeyboardInput::Function(6),
        VirtualKeyCode::F7 => KeyboardInput::Function(7),
        VirtualKeyCode::F8 => KeyboardInput::Function(8),
        VirtualKeyCode::F9 => KeyboardInput::Function(9),
        VirtualKeyCode::F10 => KeyboardInput::Function(10),
        VirtualKeyCode::F11 => KeyboardInput::Function(11),
        VirtualKeyCode::F12 => KeyboardInput::Function(12),
        VirtualKeyCode::F13 => KeyboardInput::Function(13),
        VirtualKeyCode::F14 => KeyboardInput::Function(14),
        VirtualKeyCode::F15 => KeyboardInput::Function(15),
        VirtualKeyCode::F16 => KeyboardInput::Function(16),
        VirtualKeyCode::F17 => KeyboardInput::Function(17),
        VirtualKeyCode::F18 => KeyboardInput::Function(18),
        VirtualKeyCode::F19 => KeyboardInput::Function(19),
        VirtualKeyCode::F20 => KeyboardInput::Function(20),
        VirtualKeyCode::F21 => KeyboardInput::Function(21),
        VirtualKeyCode::F22 => KeyboardInput::Function(22),
        VirtualKeyCode::F23 => KeyboardInput::Function(23),
        VirtualKeyCode::F24 => KeyboardInput::Function(24),
        _ => return None,
    };
    Some(keyboard_input)
}

fn convert_keycode(code: VirtualKeyCode, keymod: ModifiersState) -> Option<Input> {
    convert_keycode_keyboard_input(code, keymod.shift()).map(Input::Keyboard)
}

pub fn convert_event(
    event: WindowEvent,
    cell_dimensions: Dimensions<f64>,
    top_left_position: Dimensions<f64>,
    last_mouse_coord: &mut Coord,
    last_mouse_button: &mut Option<MouseButton>,
    scale_factor: f64,
    modifier_state: ModifiersState,
) -> Option<Event> {
    match event {
        WindowEvent::CloseRequested => Some(Event::Input(Input::Keyboard(chargrid_input::keys::ETX))),
        WindowEvent::Resized(physical_size) => Some(Event::Resize(physical_size.to_logical(scale_factor))),
        WindowEvent::KeyboardInput { input, .. } => {
            if let ElementState::Pressed = input.state {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    if let Some(input) = convert_keycode(virtual_keycode, modifier_state) {
                        return Some(Event::Input(input));
                    }
                }
            }
            None
        }
        WindowEvent::CursorMoved {
            position: physical_position,
            ..
        } => {
            let LogicalPosition { x, y }: LogicalPosition<f64> = physical_position.to_logical(scale_factor);
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
                GlutinMouseButton::Other(_) => return None,
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
                MouseScrollDelta::PixelDelta(LogicalPosition { x, y }) => (x as f32, y as f32),
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
