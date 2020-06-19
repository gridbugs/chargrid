use chargrid_input::GamepadInput;
use gilrs::{Button, EventType, Gilrs};

fn event_type_to_gamepad_input(event_type: EventType) -> Option<GamepadInput> {
    if let EventType::ButtonPressed(button, _code) = event_type {
        let gamepad_input = match button {
            Button::DPadUp => GamepadInput::DPadUp,
            Button::DPadRight => GamepadInput::DPadRight,
            Button::DPadDown => GamepadInput::DPadDown,
            Button::DPadLeft => GamepadInput::DPadLeft,
            Button::North => GamepadInput::North,
            Button::East => GamepadInput::East,
            Button::South => GamepadInput::South,
            Button::West => GamepadInput::West,
            Button::Start => GamepadInput::Start,
            Button::Select => GamepadInput::Select,
            Button::LeftTrigger => GamepadInput::LeftBumper,
            Button::RightTrigger => GamepadInput::RightBumper,
            _ => return None,
        };
        Some(gamepad_input)
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
                    if let Some(gamepad_input) = event_type_to_gamepad_input(event.event) {
                        return Some(gamepad_input);
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
