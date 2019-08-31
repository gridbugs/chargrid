use game::Direction;
use hashbrown::HashMap;
use prototty::input::KeyboardInput;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum AppInput {
    Move(Direction),
    Aim,
}

#[derive(Serialize, Deserialize)]
pub struct Controls {
    keys: HashMap<KeyboardInput, AppInput>,
}

impl Controls {
    pub fn default() -> Self {
        let mut keys = HashMap::new();
        keys.insert(KeyboardInput::Left, AppInput::Move(Direction::West));
        keys.insert(KeyboardInput::Right, AppInput::Move(Direction::East));
        keys.insert(KeyboardInput::Up, AppInput::Move(Direction::North));
        keys.insert(KeyboardInput::Down, AppInput::Move(Direction::South));
        keys.insert(KeyboardInput::Char('f'), AppInput::Aim);

        Self { keys }
    }

    pub fn get(&self, keyboard_input: KeyboardInput) -> Option<AppInput> {
        self.keys.get(&keyboard_input).cloned()
    }
}
