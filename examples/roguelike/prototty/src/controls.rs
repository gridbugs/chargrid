use game::{Direction, Input as GameInput};
use hashbrown::HashMap;
use prototty::input::Input as ProtottyInput;

pub struct Controls {
    controls: HashMap<ProtottyInput, GameInput>,
}

impl Controls {
    pub fn default() -> Self {
        let mut controls = HashMap::new();
        controls.insert(ProtottyInput::Left, GameInput::Move(Direction::West));
        controls.insert(ProtottyInput::Right, GameInput::Move(Direction::East));
        controls.insert(ProtottyInput::Up, GameInput::Move(Direction::North));
        controls.insert(ProtottyInput::Down, GameInput::Move(Direction::South));
        Self { controls }
    }

    pub fn get(&self, input: ProtottyInput) -> Option<GameInput> {
        self.controls.get(&input).cloned()
    }
}
