use crate::controls::Controls;
use game::{Game, ToRender};
use prototty::event_routine::common_event::*;
use prototty::event_routine::*;
use prototty::input::*;
use prototty::render::*;
use prototty_storage::{format, Storage};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

const SAVE_FILE: &'static str = "save";
const AUTO_SAVE_PERIOD: Duration = Duration::from_secs(2);

pub struct GameView;

impl<'a> View<&'a Game> for GameView {
    fn view<F: Frame, C: ColModify>(&mut self, game: &'a Game, context: ViewContext<C>, frame: &mut F) {
        let ToRender { grid } = game.to_render();
        for (coord, cell) in grid.enumerate() {
            let character = match cell.occupant {
                None => '.',
                Some(game::Occupant::Player) => '@',
                Some(game::Occupant::Wall) => '#',
            };
            let view_cell = ViewCell::new().with_character(character);
            frame.set_cell_relative(coord, 0, view_cell, context);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameInstance {
    rng: Isaac64Rng,
    game: Game,
}

pub enum RngSeed {
    Entropy,
    U64(u64),
}

impl GameInstance {
    fn new(mut rng: Isaac64Rng) -> Self {
        Self {
            game: Game::new(&mut rng),
            rng,
        }
    }
}

pub struct GameData<S: Storage> {
    instance: Option<GameInstance>,
    controls: Controls,
    storage: S,
    until_auto_save: Duration,
    rng_source: Isaac64Rng,
}

impl<S: Storage> GameData<S> {
    pub fn new(controls: Controls, storage: S, rng_seed: RngSeed) -> Self {
        let instance = storage.load(SAVE_FILE, format::Bincode).ok();
        let rng_source = match rng_seed {
            RngSeed::Entropy => Isaac64Rng::from_entropy(),
            RngSeed::U64(u64) => Isaac64Rng::seed_from_u64(u64),
        };
        Self {
            instance,
            controls,
            storage,
            until_auto_save: AUTO_SAVE_PERIOD,
            rng_source,
        }
    }
    pub fn has_instance(&self) -> bool {
        self.instance.is_some()
    }
    pub fn instantiate(&mut self) {
        let rng = Isaac64Rng::from_seed(self.rng_source.gen());
        self.instance = Some(GameInstance::new(rng));
    }
    pub fn save_instance(&mut self) {
        if let Some(instance) = self.instance.as_ref() {
            self.storage
                .store(SAVE_FILE, instance, format::Bincode)
                .expect("failed to save instance");
        } else {
            let _ = self.storage.remove(SAVE_FILE);
        }
    }
}

pub struct GameEventRoutine<S: Storage>(PhantomData<S>);

impl<S: Storage> GameEventRoutine<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub enum GameReturn {
    Pause,
}

impl<S: Storage> EventRoutine for GameEventRoutine<S> {
    type Return = GameReturn;
    type Data = GameData<S>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => {
                if input == Input::Keyboard(keys::ESCAPE) {
                    return Handled::Return(GameReturn::Pause);
                }
                if let Some(instance) = data.instance.as_mut() {
                    let maybe_game_input = data.controls.get(input);
                    if let Some(game_input) = maybe_game_input {
                        instance.game.handle_input(game_input);
                    }
                }
                Handled::Continue(s)
            }
            CommonEvent::Frame(period) => {
                if let Some(until_auto_save) = data.until_auto_save.checked_sub(period) {
                    data.until_auto_save = until_auto_save;
                } else {
                    data.save_instance();
                    data.until_auto_save = AUTO_SAVE_PERIOD;
                }
                Handled::Continue(s)
            }
        })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.instance.as_ref() {
            view.view(&instance.game, context, frame);
        }
    }
}
