use crate::controls::{AppInput, Controls};
pub use game::Input as GameInput;
use game::{CardinalDirection, CellVisibility, Game, Layer, Tile, ToRenderEntity, VisibilityGrid};
use line_2d::{Config as LineConfig, LineSegment};
use prototty::event_routine::common_event::*;
use prototty::event_routine::*;
use prototty::input::*;
use prototty::render::*;
use prototty_audio::AudioPlayer;
use prototty_storage::{format, Storage};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

const AUTO_SAVE_PERIOD: Duration = Duration::from_secs(2);
const AIM_UI_DEPTH: i8 = 3;
const PLAYER_OFFSET: Coord = Coord::new(16, 16);
const GAME_WINDOW_SIZE: Size = Size::new_u16((PLAYER_OFFSET.x as u16 * 2) + 1, (PLAYER_OFFSET.y as u16 * 2) + 1);

pub enum InjectedInput {
    Fire(ScreenCoord),
}

pub struct GameView {
    last_offset: Coord,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            last_offset: Coord::new(0, 0),
        }
    }
    pub fn absolute_coord_to_game_relative_screen_coord(&self, coord: Coord) -> ScreenCoord {
        ScreenCoord(coord - self.last_offset)
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

#[derive(Clone, Copy)]
pub struct ScreenCoord(Coord);

#[derive(Clone, Copy)]
struct GameCoord(Coord);

#[derive(Clone, Copy)]
struct PlayerCoord(Coord);

impl GameCoord {
    fn of_player(game: &Game) -> Self {
        Self(game.player_coord())
    }
}

struct GameCoordToScreenCoord {
    game_coord: GameCoord,
    player_coord: GameCoord,
}

impl GameCoordToScreenCoord {
    fn compute(self) -> ScreenCoord {
        ScreenCoord(self.game_coord.0 - self.player_coord.0 + PLAYER_OFFSET)
    }
}

struct ScreenCoordToGameCoord {
    screen_coord: ScreenCoord,
    player_coord: GameCoord,
}

impl ScreenCoordToGameCoord {
    fn compute(self) -> GameCoord {
        GameCoord(self.screen_coord.0 + self.player_coord.0 - PLAYER_OFFSET)
    }
}

fn render_entity<F: Frame, C: ColModify>(
    to_render_entity: &ToRenderEntity,
    game: &Game,
    visibility_grid: &VisibilityGrid,
    player_coord: GameCoord,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let entity_coord = GameCoord(to_render_entity.coord);
    if let CellVisibility::VisibleWithLightColour(light_colour) = visibility_grid.cell_visibility(entity_coord.0) {
        if light_colour == Rgb24::new(0, 0, 0) {
            return;
        }
        let screen_coord = GameCoordToScreenCoord {
            game_coord: entity_coord,
            player_coord,
        }
        .compute();
        if !screen_coord.0.is_valid(GAME_WINDOW_SIZE) {
            return;
        }
        let depth = layer_depth(to_render_entity.layer);
        let mut view_cell = match to_render_entity.tile {
            Tile::Player => ViewCell::new().with_character('@'),
            Tile::Floor => ViewCell::new().with_character('.').with_background(Rgb24::new(0, 0, 0)),
            Tile::Carpet => ViewCell::new()
                .with_character('.')
                .with_background(Rgb24::new(0, 0, 127)),
            Tile::Wall => if game.contains_wall(entity_coord.0 + Coord::new(0, 1)) {
                ViewCell::new().with_character('█')
            } else {
                ViewCell::new().with_character('▀')
            }
            .with_foreground(Rgb24::new(255, 255, 255))
            .with_background(Rgb24::new(127, 127, 127)),
            Tile::Bullet => ViewCell::new().with_character('*'),
            Tile::Smoke => {
                if let Some(fade) = to_render_entity.fade {
                    frame.blend_cell_background_relative(
                        screen_coord.0,
                        depth,
                        Rgb24::new_grey(187).normalised_mul(light_colour),
                        (255 - fade) / 10,
                        blend_mode::LinearInterpolate,
                        context,
                    )
                }
                return;
            }
            Tile::ExplosionFlame => {
                if let Some(fade) = to_render_entity.fade {
                    if let Some(colour_hint) = to_render_entity.colour_hint {
                        let quad_fade = (((fade as u16) * (fade as u16)) / 256) as u8;
                        let start_colour = colour_hint;
                        let end_colour = Rgb24::new(255, 0, 0);
                        let interpolated_colour = start_colour.linear_interpolate(end_colour, quad_fade);
                        let lit_colour = interpolated_colour.normalised_mul(light_colour);
                        frame.blend_cell_background_relative(
                            screen_coord.0,
                            depth,
                            lit_colour,
                            (255 - fade) / 1,
                            blend_mode::LinearInterpolate,
                            context,
                        )
                    }
                }
                return;
            }
        };
        if let Some(foreground) = view_cell.style.foreground.as_mut() {
            *foreground = foreground.normalised_mul(light_colour);
        }
        if let Some(background) = view_cell.style.background.as_mut() {
            *background = background.normalised_mul(light_colour);
        }
        frame.set_cell_relative(screen_coord.0, depth, view_cell, context);
    }
}

impl<'a> View<&'a Game> for GameView {
    fn view<F: Frame, C: ColModify>(&mut self, game: &'a Game, context: ViewContext<C>, frame: &mut F) {
        let player_coord = GameCoord::of_player(&game);
        let visibility_grid = game.visibility_grid();
        for to_render_entity in game.to_render_entities() {
            render_entity(&to_render_entity, game, visibility_grid, player_coord, context, frame);
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

pub struct GameData<S: Storage, A: AudioPlayer> {
    instance: Option<GameInstance>,
    controls: Controls,
    rng_source: Isaac64Rng,
    last_aim_with_mouse: bool,
    storage_wrapper: StorageWrapper<S>,
    _audio_player: A,
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

impl<S: Storage, A: AudioPlayer> GameData<S, A> {
    pub fn new(controls: Controls, storage: S, save_key: String, audio_player: A, rng_seed: RngSeed) -> Self {
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
            last_aim_with_mouse: false,
            storage_wrapper,
            _audio_player: audio_player,
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
    pub fn game(&self) -> Result<&Game, NoGameInstance> {
        self.instance.as_ref().map(|i| &i.game).ok_or(NoGameInstance)
    }
    pub fn initial_aim_coord(&self, screen_coord_of_mouse: ScreenCoord) -> Result<ScreenCoord, NoGameInstance> {
        if let Some(instance) = self.instance.as_ref() {
            if self.last_aim_with_mouse {
                Ok(screen_coord_of_mouse)
            } else {
                let player_coord = GameCoord::of_player(&instance.game);
                let screen_coord = GameCoordToScreenCoord {
                    game_coord: player_coord,
                    player_coord,
                }
                .compute();
                Ok(screen_coord)
            }
        } else {
            Err(NoGameInstance)
        }
    }
}

pub struct NoGameInstance;

pub struct AimEventRoutine<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
    screen_coord: ScreenCoord,
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

impl<S: Storage, A: AudioPlayer> AimEventRoutine<S, A> {
    pub fn new(screen_coord: ScreenCoord) -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
            screen_coord,
            duration: Duration::from_millis(0),
            blink: Blink {
                cycle_length: Duration::from_millis(500),
                min_alpha: 64,
                max_alpha: 187,
            },
        }
    }
}

impl<S: Storage, A: AudioPlayer> EventRoutine for AimEventRoutine<S, A> {
    type Return = Option<ScreenCoord>;
    type Data = GameData<S, A>;
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
                    Aim::KeyboardFinalise => Handled::Return(Some(s.screen_coord)),
                    Aim::KeyboardDirection(direction) => {
                        s.screen_coord.0 += direction.coord();
                        Handled::Continue(s)
                    }
                    Aim::Mouse { coord, press } => {
                        s.screen_coord = view.absolute_coord_to_game_relative_screen_coord(coord);
                        if press {
                            *last_aim_with_mouse = true;
                            Handled::Return(Some(s.screen_coord))
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
            let player_coord = GameCoord::of_player(&instance.game);
            let screen_coord = self.screen_coord;
            let game_coord = ScreenCoordToGameCoord {
                screen_coord,
                player_coord,
            }
            .compute();
            if game_coord.0 != player_coord.0 {
                for node in LineSegment::new(player_coord.0, game_coord.0).config_node_iter(LineConfig {
                    exclude_start: true,
                    exclude_end: true,
                }) {
                    let screen_coord = GameCoordToScreenCoord {
                        player_coord,
                        game_coord: GameCoord(node.coord),
                    }
                    .compute();
                    if !screen_coord.0.is_valid(GAME_WINDOW_SIZE) {
                        break;
                    }
                    frame.blend_cell_background_relative(
                        screen_coord.0,
                        AIM_UI_DEPTH,
                        Rgb24::new(255, 0, 0),
                        127,
                        blend_mode::LinearInterpolate,
                        context,
                    );
                }
            }
            if screen_coord.0.is_valid(GAME_WINDOW_SIZE) {
                let alpha = self.blink.alpha(self.duration);
                frame.blend_cell_background_relative(
                    screen_coord.0,
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

pub struct GameEventRoutine<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
    injected_inputs: Vec<InjectedInput>,
}

impl<S: Storage, A: AudioPlayer> GameEventRoutine<S, A> {
    pub fn new() -> Self {
        Self::new_injecting_inputs(Vec::new())
    }
    pub fn new_injecting_inputs(injected_inputs: Vec<InjectedInput>) -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
            injected_inputs,
        }
    }
}

pub enum GameReturn {
    Pause,
    Aim,
}

impl<S: Storage, A: AudioPlayer> EventRoutine for GameEventRoutine<S, A> {
    type Return = GameReturn;
    type Data = GameData<S, A>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(mut self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let storage_wrapper = &mut data.storage_wrapper;
        if let Some(instance) = data.instance.as_mut() {
            for injected_input in self.injected_inputs.drain(..) {
                match injected_input {
                    InjectedInput::Fire(screen_coord) => {
                        let player_coord = GameCoord::of_player(&instance.game);
                        let GameCoord(raw_game_coord) = ScreenCoordToGameCoord {
                            screen_coord,
                            player_coord,
                        }
                        .compute();
                        instance.game.handle_input(GameInput::Fire(raw_game_coord));
                    }
                }
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
