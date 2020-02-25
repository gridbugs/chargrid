use game::CardinalDirection;
use hashbrown::HashMap;
use prototty::input::KeyboardInput;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum AppInput {
    Move(CardinalDirection),
    Aim,
    Wait,
    Map,
}

#[derive(Serialize, Deserialize)]
pub struct Controls {
    keys: HashMap<KeyboardInput, AppInput>,
}

impl Controls {
    pub fn default() -> Self {
        let mut keys = HashMap::new();
        keys.insert(KeyboardInput::Left, AppInput::Move(CardinalDirection::West));
        keys.insert(KeyboardInput::Right, AppInput::Move(CardinalDirection::East));
        keys.insert(KeyboardInput::Up, AppInput::Move(CardinalDirection::North));
        keys.insert(KeyboardInput::Down, AppInput::Move(CardinalDirection::South));
        keys.insert(KeyboardInput::Char('f'), AppInput::Aim);
        keys.insert(KeyboardInput::Char(' '), AppInput::Wait);
        keys.insert(KeyboardInput::Char('m'), AppInput::Map);
        Self { keys }
    }

    pub fn get(&self, keyboard_input: KeyboardInput) -> Option<AppInput> {
        self.keys.get(&keyboard_input).cloned()
    }
}
