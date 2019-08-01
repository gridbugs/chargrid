use game::{Direction, Input as GameInput};
use hashbrown::HashMap;
use prototty::input::{Input, KeyboardInput};

pub struct Controls {
    keys: HashMap<KeyboardInput, GameInput>,
}

impl Controls {
    pub fn default() -> Self {
        let mut keys = HashMap::new();
        keys.insert(KeyboardInput::Left, GameInput::Move(Direction::West));
        keys.insert(KeyboardInput::Right, GameInput::Move(Direction::East));
        keys.insert(KeyboardInput::Up, GameInput::Move(Direction::North));
        keys.insert(KeyboardInput::Down, GameInput::Move(Direction::South));
        Self { keys }
    }

    pub fn get(&self, input: Input) -> Option<GameInput> {
        match input {
            Input::Keyboard(keyboard_input) => self.keys.get(&keyboard_input).cloned(),
            Input::Mouse(_) => None,
        }
    }
}
