use crate::controls::{AppInput, Controls};
pub use game::Input as GameInput;
use game::{Direction, Game, ToRender};
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

const AUTO_SAVE_PERIOD: Duration = Duration::from_secs(2);

pub struct GameView {
    last_offset: Coord,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            last_offset: Coord::new(0, 0),
        }
    }
}

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
        self.last_offset = context.offset;
    }
}

#[derive(Serialize, Deserialize)]
struct GameInstance {
    rng: Isaac64Rng,
    game: Game,
}

#[derive(Clone)]
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
    rng_source: Isaac64Rng,
    storage_wrapper: StorageWrapper<S>,
}

struct StorageWrapper<S: Storage> {
    storage: S,
    save_key: String,
    until_auto_save: Duration,
}

impl<S: Storage> StorageWrapper<S> {
    pub fn save_instance(&mut self, instance: &GameInstance) {
        self.storage
            .store(&self.save_key, instance, format::Bincode)
            .expect("failed to save instance");
    }
    pub fn clear_instance(&mut self) {
        let _ = self.storage.remove(&self.save_key);
    }
    pub fn autosave_tick(&mut self, instance: &GameInstance, since_previous: Duration) {
        if let Some(remaining) = self.until_auto_save.checked_sub(since_previous) {
            self.until_auto_save = remaining;
        } else {
            self.save_instance(instance);
            self.until_auto_save = AUTO_SAVE_PERIOD;
        }
    }
}

impl<S: Storage> GameData<S> {
    pub fn new(controls: Controls, storage: S, save_key: String, rng_seed: RngSeed) -> Self {
        let instance = storage.load(&save_key, format::Bincode).ok();
        let rng_source = match rng_seed {
            RngSeed::Entropy => Isaac64Rng::from_entropy(),
            RngSeed::U64(u64) => Isaac64Rng::seed_from_u64(u64),
        };
        let storage_wrapper = StorageWrapper {
            storage,
            save_key,
            until_auto_save: AUTO_SAVE_PERIOD,
        };
        Self {
            instance,
            controls,
            rng_source,
            storage_wrapper,
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
            self.storage_wrapper.save_instance(instance);
        } else {
            self.storage_wrapper.clear_instance();
        }
    }
    pub fn clear_instance(&mut self) {
        self.instance = None;
        self.storage_wrapper.clear_instance();
    }
    pub fn game(&self) -> Option<&Game> {
        self.instance.as_ref().map(|i| &i.game)
    }
}

pub struct AimEventRoutine<S: Storage> {
    s: PhantomData<S>,
    coord: Coord,
}

impl<S: Storage> AimEventRoutine<S> {
    pub fn new(coord: Coord) -> Self {
        Self { s: PhantomData, coord }
    }
}

impl<S: Storage> EventRoutine for AimEventRoutine<S> {
    type Return = Option<Coord>;
    type Data = GameData<S>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(self, _data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |mut s, event| {
            let direction = match event {
                CommonEvent::Input(input) => match input {
                    Input::Keyboard(KeyboardInput::Up) => Direction::North,
                    Input::Keyboard(KeyboardInput::Down) => Direction::South,
                    Input::Keyboard(KeyboardInput::Left) => Direction::West,
                    Input::Keyboard(KeyboardInput::Right) => Direction::East,
                    Input::Mouse(MouseInput::MouseMove { coord, .. }) => {
                        s.coord = coord - view.last_offset;
                        return Handled::Continue(s);
                    }
                    Input::Mouse(MouseInput::MousePress { coord, .. }) => {
                        s.coord = coord - view.last_offset;
                        return Handled::Return(Some(s.coord));
                    }
                    Input::Keyboard(keys::RETURN) => return Handled::Return(Some(s.coord)),
                    Input::Keyboard(keys::ESCAPE) => return Handled::Return(None),
                    _ => return Handled::Continue(s),
                },
                CommonEvent::Frame(_) => return Handled::Continue(s),
            };
            s.coord += direction.coord();
            Handled::Continue(s)
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
        frame.set_cell_relative(
            self.coord,
            1,
            ViewCell::new().with_background(Rgb24::new(255, 0, 0)),
            context,
        );
    }
}

pub struct GameEventRoutine<S: Storage> {
    s: PhantomData<S>,
    injected_inputs: Vec<GameInput>,
}

impl<S: Storage> GameEventRoutine<S> {
    pub fn new() -> Self {
        Self::new_injecting_inputs(Vec::new())
    }
    pub fn new_injecting_inputs(injected_inputs: Vec<GameInput>) -> Self {
        Self {
            s: PhantomData,
            injected_inputs,
        }
    }
}

pub enum GameReturn {
    Pause,
    Aim,
}

impl<S: Storage> EventRoutine for GameEventRoutine<S> {
    type Return = GameReturn;
    type Data = GameData<S>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(mut self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let storage_wrapper = &mut data.storage_wrapper;
        if let Some(instance) = data.instance.as_mut() {
            for game_input in self.injected_inputs.drain(..) {
                instance.game.handle_input(game_input);
            }
            let controls = &data.controls;
            event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
                CommonEvent::Input(input) => {
                    if input == Input::Keyboard(keys::ESCAPE) {
                        return Handled::Return(GameReturn::Pause);
                    }
                    if let Some(app_input) = controls.get(input) {
                        match app_input {
                            AppInput::GameInput(game_input) => instance.game.handle_input(game_input),
                            AppInput::Aim => return Handled::Return(GameReturn::Aim),
                        }
                    }
                    Handled::Continue(s)
                }
                CommonEvent::Frame(period) => {
                    storage_wrapper.autosave_tick(instance, period);
                    Handled::Continue(s)
                }
            })
        } else {
            storage_wrapper.clear_instance();
            Handled::Continue(self)
        }
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
