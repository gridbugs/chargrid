use crate::controls::{AppInput, Controls};
pub use game::Input as GameInput;
use game::{CardinalDirection, Game, Layer, Tile, ToRenderEntity};
use line_2d::{Config as LineConfig, LineSegment};
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
    pub fn absolute_coord_to_game_coord(&self, coord: Coord) -> Coord {
        coord - self.last_offset
    }
}

fn layer_depth(layer: Layer) -> i8 {
    match layer {
        Layer::Floor => 0,
        Layer::Feature => 1,
        Layer::Character => 2,
        Layer::Particle => 3,
    }
}

const AIM_UI_DEPTH: i8 = 3;

fn render_entity<F: Frame, C: ColModify>(to_render_entity: &ToRenderEntity, context: ViewContext<C>, frame: &mut F) {
    let depth = layer_depth(to_render_entity.layer);
    let coord = to_render_entity.coord;
    let view_cell = match to_render_entity.tile {
        Tile::Player => ViewCell::new().with_character('@'),
        Tile::Floor => ViewCell::new().with_character('.').with_background(Rgb24::new(0, 0, 0)),
        Tile::Carpet => ViewCell::new()
            .with_character('.')
            .with_background(Rgb24::new(0, 0, 127)),
        Tile::Wall => ViewCell::new().with_character('#'),
        Tile::Bullet => ViewCell::new().with_character('*'),
        Tile::Smoke => {
            if let Some(fade) = to_render_entity.fade {
                frame.blend_cell_background_relative(
                    coord,
                    depth,
                    Rgb24::new_grey(187),
                    127 - (fade / 2),
                    blend_mode::LinearInterpolate,
                    context,
                )
            }
            return;
        }
    };
    frame.set_cell_relative(coord, depth, view_cell, context);
}

impl<'a> View<&'a Game> for GameView {
    fn view<F: Frame, C: ColModify>(&mut self, game: &'a Game, context: ViewContext<C>, frame: &mut F) {
        for to_render_entity in game.to_render_entities() {
            render_entity(&to_render_entity, context, frame);
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
    last_aim_with_mouse: bool,
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
            last_aim_with_mouse: false,
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
    pub fn game(&self) -> Result<&Game, NoGameInstnace> {
        self.instance.as_ref().map(|i| &i.game).ok_or(NoGameInstnace)
    }
    pub fn initial_aim_coord(&self, mouse_coord: Coord) -> Result<Coord, NoGameInstnace> {
        if let Some(instance) = self.instance.as_ref() {
            if self.last_aim_with_mouse {
                Ok(mouse_coord)
            } else {
                Ok(instance.game.player_coord())
            }
        } else {
            Err(NoGameInstnace)
        }
    }
}

pub struct NoGameInstnace;

pub struct AimEventRoutine<S: Storage> {
    s: PhantomData<S>,
    coord: Coord,
    duration: Duration,
    blink: Blink,
}

struct Blink {
    cycle_length: Duration,
    min_alpha: u8,
    max_alpha: u8,
}

impl Blink {
    fn intensity(&self, duration: Duration) -> u8 {
        let cycle_length_micros = self.cycle_length.as_micros();
        let duration_micros = duration.as_micros();
        let progress_through_cycle_micros = duration_micros % cycle_length_micros;
        let scaled_progress = (progress_through_cycle_micros * 512) / cycle_length_micros;
        if scaled_progress < 256 {
            scaled_progress as u8
        } else {
            (511 - scaled_progress) as u8
        }
    }
    fn alpha(&self, duration: Duration) -> u8 {
        let intensity = self.intensity(duration);
        let delta = self.max_alpha - self.min_alpha;
        let offset = ((delta as u16 * intensity as u16) / 255 as u16) as u8;
        self.min_alpha + offset
    }
}

impl<S: Storage> AimEventRoutine<S> {
    pub fn new(coord: Coord) -> Self {
        Self {
            s: PhantomData,
            coord,
            duration: Duration::from_millis(0),
            blink: Blink {
                cycle_length: Duration::from_millis(500),
                min_alpha: 64,
                max_alpha: 187,
            },
        }
    }
}

impl<S: Storage> EventRoutine for AimEventRoutine<S> {
    type Return = Option<Coord>;
    type Data = GameData<S>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        enum Aim {
            Mouse { coord: Coord, press: bool },
            KeyboardDirection(CardinalDirection),
            KeyboardFinalise,
            Cancel,
            Ignore,
            Frame(Duration),
        }
        let last_aim_with_mouse = &mut data.last_aim_with_mouse;
        let controls = &data.controls;
        if let Some(instance) = data.instance.as_mut() {
            event_or_peek_with_handled(event_or_peek, self, |mut s, event| {
                *last_aim_with_mouse = false;
                let aim = match event {
                    CommonEvent::Input(input) => match input {
                        Input::Keyboard(keyboard_input) => {
                            if let Some(app_input) = controls.get(keyboard_input) {
                                match app_input {
                                    AppInput::Aim => Aim::KeyboardFinalise,
                                    AppInput::Move(direction) => Aim::KeyboardDirection(direction),
                                }
                            } else {
                                match keyboard_input {
                                    keys::RETURN => Aim::KeyboardFinalise,
                                    keys::ESCAPE => Aim::Cancel,
                                    _ => Aim::Ignore,
                                }
                            }
                        }
                        Input::Mouse(mouse_input) => match mouse_input {
                            MouseInput::MouseMove { coord, .. } => Aim::Mouse { coord, press: false },
                            MouseInput::MousePress { coord, .. } => Aim::Mouse { coord, press: true },
                            _ => Aim::Ignore,
                        },
                    },
                    CommonEvent::Frame(since_last) => Aim::Frame(since_last),
                };
                match aim {
                    Aim::KeyboardFinalise => Handled::Return(Some(s.coord)),
                    Aim::KeyboardDirection(direction) => {
                        s.coord += direction.coord();
                        Handled::Continue(s)
                    }
                    Aim::Mouse { coord, press } => {
                        s.coord = view.absolute_coord_to_game_coord(coord);
                        if press {
                            *last_aim_with_mouse = true;
                            Handled::Return(Some(s.coord))
                        } else {
                            Handled::Continue(s)
                        }
                    }
                    Aim::Cancel => Handled::Return(None),
                    Aim::Ignore => Handled::Continue(s),
                    Aim::Frame(since_last) => {
                        instance.game.handle_tick(since_last);
                        s.duration += since_last;
                        Handled::Continue(s)
                    }
                }
            })
        } else {
            Handled::Return(None)
        }
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.instance.as_ref() {
            view.view(&instance.game, context, frame);
            let player_coord = instance.game.player_coord();
            if self.coord != player_coord {
                for node in LineSegment::new(player_coord, self.coord).config_node_iter(LineConfig {
                    exclude_start: true,
                    exclude_end: true,
                }) {
                    if !node.coord.is_valid(instance.game.world_size()) {
                        break;
                    }
                    frame.blend_cell_background_relative(
                        node.coord,
                        AIM_UI_DEPTH,
                        Rgb24::new(255, 0, 0),
                        127,
                        blend_mode::LinearInterpolate,
                        context,
                    );
                }
            }
            if self.coord.is_valid(instance.game.world_size()) {
                let alpha = self.blink.alpha(self.duration);
                frame.blend_cell_background_relative(
                    self.coord,
                    AIM_UI_DEPTH,
                    Rgb24::new(255, 0, 0),
                    alpha,
                    blend_mode::LinearInterpolate,
                    context,
                );
            }
        }
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
                    match input {
                        Input::Keyboard(keyboard_input) => {
                            if keyboard_input == keys::ESCAPE {
                                return Handled::Return(GameReturn::Pause);
                            }
                            if !instance.game.is_gameplay_blocked() {
                                if let Some(app_input) = controls.get(keyboard_input) {
                                    match app_input {
                                        AppInput::Move(direction) => {
                                            instance.game.handle_input(GameInput::Walk(direction))
                                        }
                                        AppInput::Aim => return Handled::Return(GameReturn::Aim),
                                    }
                                }
                            }
                        }
                        Input::Mouse(_) => (),
                    }
                    Handled::Continue(s)
                }
                CommonEvent::Frame(period) => {
                    instance.game.handle_tick(period);
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
