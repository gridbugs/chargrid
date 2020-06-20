use chargrid_input::{GamepadButton, GamepadInput};
use gilrs::{Button, EventType, Gilrs};

fn event_type_to_gamepad_button(event_type: EventType) -> Option<GamepadButton> {
    if let EventType::ButtonPressed(button, _code) = event_type {
        let gamepad_button = match button {
            Button::DPadUp => GamepadButton::DPadUp,
            Button::DPadRight => GamepadButton::DPadRight,
            Button::DPadDown => GamepadButton::DPadDown,
            Button::DPadLeft => GamepadButton::DPadLeft,
            Button::North => GamepadButton::North,
            Button::East => GamepadButton::East,
            Button::South => GamepadButton::South,
            Button::West => GamepadButton::West,
            Button::Start => GamepadButton::Start,
            Button::Select => GamepadButton::Select,
            Button::LeftTrigger => GamepadButton::LeftBumper,
            Button::RightTrigger => GamepadButton::RightBumper,
            _ => return None,
        };
        Some(gamepad_button)
    } else {
        None
    }
}

pub struct GamepadContext {
    gilrs: Option<Gilrs>,
}

impl GamepadContext {
    pub fn new() -> Self {
        let gilrs = Gilrs::new();
        match gilrs {
            Err(ref err) => log::error!("Couldn't connect to controller: {}", err),
            Ok(ref gilrs) => {
                for (_id, gamepad) in gilrs.gamepads() {
                    log::info!("{} is {:?}", gamepad.name(), gamepad.power_info());
                }
            }
        }
        Self { gilrs: gilrs.ok() }
    }

    pub fn drain_input(&mut self) -> GamepadDrainInput {
        GamepadDrainInput {
            gilrs: self.gilrs.as_mut(),
        }
    }
}

pub struct GamepadDrainInput<'a> {
    gilrs: Option<&'a mut Gilrs>,
}

impl<'a> Iterator for GamepadDrainInput<'a> {
    type Item = GamepadInput;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut gilrs) = self.gilrs {
            loop {
                if let Some(event) = gilrs.next_event() {
                    if let Some(button) = event_type_to_gamepad_button(event.event) {
                        let id_usize: usize = event.id.into();
                        let id = id_usize as u64;
                        return Some(GamepadInput { button, id });
                    }
                } else {
                    return None;
                }
            }
        } else {
            None
        }
    }
}
