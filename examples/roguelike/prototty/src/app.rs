use crate::controls::Controls;
use crate::game::{GameData, GameEventRoutine, GameView};
use common_event::*;
use event_routine::*;
use prototty::*;

pub struct AppData {
    game: GameData,
}

impl AppData {
    pub fn new(controls: Controls) -> Self {
        Self {
            game: GameData::new(controls),
        }
    }
}

pub struct AppView {
    game: GameView,
}

impl AppView {
    pub fn new() -> Self {
        Self { game: GameView }
    }
}

struct SelectGame;
impl ViewSelector for SelectGame {
    type ViewInput = AppView;
    type ViewOutput = GameView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.game
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.game
    }
}
impl DataSelector for SelectGame {
    type DataInput = AppData;
    type DataOutput = GameData;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.game
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.game
    }
}
impl Selector for SelectGame {}

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine.select(SelectGame).return_on_exit(|_| ())
}
