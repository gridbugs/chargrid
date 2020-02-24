use crate::audio::{Audio, AudioTable};
use crate::controls::{AppInput, Controls};
use crate::depth;
use crate::frontend::Frontend;
use direction::{CardinalDirection, Direction};
use game::{
    CellVisibility, CharacterInfo, ExternalEvent, Game, GameControlFlow, Layer, Tile, ToRenderEntity, VisibilityGrid,
};
pub use game::{Config as GameConfig, Input as GameInput, Omniscient};
use line_2d::{Config as LineConfig, LineSegment};
use prototty::event_routine::common_event::*;
use prototty::event_routine::*;
use prototty::input::*;
use prototty::render::*;
use prototty::text::*;
use prototty_audio::{AudioPlayer, AudioProperties};
use prototty_storage::{format, Storage};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::Duration;

const AIM_UI_DEPTH: i8 = depth::GAME_MAX;
const PLAYER_OFFSET: Coord = Coord::new(22, 18);
const GAME_WINDOW_SIZE: Size = Size::new_u16((PLAYER_OFFSET.x as u16 * 2) + 1, (PLAYER_OFFSET.y as u16 * 2) + 1);
const STORAGE_FORMAT: format::Bincode = format::Bincode;

#[derive(Serialize, Deserialize, Clone, Copy)]
struct ScreenShake {
    remaining_frames: u8,
    direction: Direction,
}

impl ScreenShake {
    fn coord(&self) -> Coord {
        self.direction.coord()
    }
    fn next(self) -> Option<Self> {
        self.remaining_frames.checked_sub(1).map(|remaining_frames| Self {
            remaining_frames,
            direction: self.direction,
        })
    }
}

struct EffectContext<'a, A: AudioPlayer> {
    rng: &'a mut Isaac64Rng,
    screen_shake: &'a mut Option<ScreenShake>,
    audio_player: &'a A,
    audio_table: &'a AudioTable<A>,
    player_coord: GameCoord,
}

impl<'a, A: AudioPlayer> EffectContext<'a, A> {
    fn next_frame(&mut self) {
        *self.screen_shake = self.screen_shake.and_then(|screen_shake| screen_shake.next());
    }
    fn play_audio(&self, audio: Audio, properties: AudioProperties) {
        log::info!("Playing sound: {:?} {:?}", audio, properties);
        let sound = self.audio_table.get(audio);
        self.audio_player.play(&sound, properties);
    }
    fn handle_event(&mut self, event: ExternalEvent) {
        match event {
            ExternalEvent::Explosion(coord) => {
                let direction: Direction = self.rng.gen();
                *self.screen_shake = Some(ScreenShake {
                    remaining_frames: 2,
                    direction,
                });
                const BASE_VOLUME: f32 = 50.;
                let distance_squared = (self.player_coord.0 - coord).magnitude2();
                let volume = (BASE_VOLUME / (distance_squared as f32).max(1.)).min(1.);
                let properties = AudioProperties::default().with_volume(volume);
                self.play_audio(Audio::Explosion, properties);
            }
        }
    }
}

pub enum InjectedInput {
    Fire(ScreenCoord),
}

mod status_line_parts {
    pub const HP_TITLE: usize = 0;
    pub const HP_CURRENT: usize = 1;
    pub const HP_MAX: usize = 2;
    pub const N: usize = 3;
}

struct StatusLineView {
    buffer: Vec<RichTextPartOwned>,
}

impl StatusLineView {
    fn new() -> Self {
        let mut buffer = Vec::new();
        for _ in 0..status_line_parts::N {
            buffer.push(RichTextPartOwned {
                text: String::new(),
                style: Style::new(),
            });
        }
        Self { buffer }
    }
    fn update(&mut self, player_info: &CharacterInfo) {
        use std::fmt::Write;
        {
            let hp_title = &mut self.buffer[status_line_parts::HP_TITLE];
            hp_title.style = Style::new().with_foreground(Rgb24::new(255, 255, 255));
            hp_title.text.clear();
            write!(&mut hp_title.text, "HP: ").unwrap();
        }
        {
            let hp_current_colour = if player_info.hit_points.current < player_info.hit_points.max / 4 {
                Rgb24::new(255, 0, 0)
            } else {
                Rgb24::new(255, 255, 255)
            };
            let hp_current = &mut self.buffer[status_line_parts::HP_CURRENT];
            hp_current.style = Style::new().with_foreground(hp_current_colour);
            hp_current.text.clear();
            write!(&mut hp_current.text, "{}", player_info.hit_points.current).unwrap();
        }
        {
            let hp_max = &mut self.buffer[status_line_parts::HP_MAX];
            hp_max.style = Style::new().with_foreground(Rgb24::new(255, 255, 255));
            hp_max.text.clear();
            write!(&mut hp_max.text, "/{}", player_info.hit_points.max).unwrap();
        }
    }
}

pub struct GameView {
    last_offset: Coord,
    status_line_view: StatusLineView,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            last_offset: Coord::new(0, 0),
            status_line_view: StatusLineView::new(),
        }
    }
    pub fn absolute_coord_to_game_relative_screen_coord(&self, coord: Coord) -> ScreenCoord {
        ScreenCoord(coord - self.last_offset)
    }
    pub fn view<F: Frame, C: ColModify>(
        &mut self,
        game_to_render: GameToRender,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        let game = &game_to_render.game;
        let player_info = game.player_info();
        let player_coord = GameCoord::of_player(player_info);
        let visibility_grid = game.visibility_grid();
        let offset = game_to_render
            .screen_shake
            .as_ref()
            .map(|d| d.coord())
            .unwrap_or_else(|| Coord::new(0, 0));
        for to_render_entity in game.to_render_entities() {
            render_entity(
                game_to_render.status,
                &to_render_entity,
                game,
                visibility_grid,
                player_coord,
                offset,
                context,
                frame,
            );
        }
        self.last_offset = context.offset;
        self.status_line_view.update(player_info);
        RichTextViewSingleLine.view(
            self.status_line_view.buffer.iter().map(|s| s.as_rich_text_part()),
            context.add_offset(Coord::new(1, 1 + GAME_WINDOW_SIZE.height() as i32)),
            frame,
        );
    }
}

fn layer_depth(layer: Option<Layer>) -> i8 {
    if let Some(layer) = layer {
        match layer {
            Layer::Floor => 0,
            Layer::Feature => 1,
            Layer::Character => 2,
        }
    } else {
        depth::GAME_MAX - 1
    }
}

#[derive(Clone, Copy)]
pub struct ScreenCoord(Coord);

#[derive(Clone, Copy)]
struct GameCoord(Coord);

#[derive(Clone, Copy)]
struct PlayerCoord(Coord);

impl GameCoord {
    fn of_player(player_info: &CharacterInfo) -> Self {
        Self(player_info.coord)
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
    game_status: GameStatus,
    to_render_entity: &ToRenderEntity,
    game: &Game,
    visibility_grid: &VisibilityGrid,
    player_coord: GameCoord,
    offset: Coord,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let entity_coord = GameCoord(to_render_entity.coord);
    let light_colour = if let GameStatus::Over = game_status {
        Rgb24::new(255, 0, 0)
    } else {
        match visibility_grid.cell_visibility(entity_coord.0) {
            CellVisibility::VisibleWithLightColour(Some(light_colour)) => {
                if to_render_entity.ignore_lighting {
                    Rgb24::new(255, 255, 255)
                } else {
                    light_colour
                }
            }
            CellVisibility::VisibleWithLightColour(None) => {
                if to_render_entity.ignore_lighting {
                    Rgb24::new(255, 255, 255)
                } else {
                    return;
                }
            }
            CellVisibility::NotVisible => return,
        }
    };
    if game_status == GameStatus::Playing && light_colour == Rgb24::new(0, 0, 0) {
        return;
    }
    let screen_coord = GameCoordToScreenCoord {
        game_coord: entity_coord,
        player_coord: GameCoord(player_coord.0 + offset),
    }
    .compute();
    if !screen_coord.0.is_valid(GAME_WINDOW_SIZE) {
        return;
    }
    let depth = layer_depth(to_render_entity.layer);
    let mut view_cell = match to_render_entity.tile {
        Tile::Player => ViewCell::new()
            .with_character('@')
            .with_bold(true)
            .with_foreground(Rgb24::new(255, 255, 255)),
        Tile::FormerHuman => ViewCell::new()
            .with_character('f')
            .with_foreground(Rgb24::new(255, 0, 0)),
        Tile::Human => ViewCell::new()
            .with_character('h')
            .with_foreground(Rgb24::new(0, 255, 255)),
        Tile::Floor => ViewCell::new()
            .with_character('.')
            .with_background(Rgb24::new(63, 63, 63))
            .with_foreground(Rgb24::new(127, 127, 127)),
        Tile::Carpet => ViewCell::new()
            .with_character('.')
            .with_background(Rgb24::new(127, 0, 0))
            .with_foreground(Rgb24::new(127, 127, 127)),
        Tile::Star => {
            let foreground_colour = to_render_entity.colour_hint.unwrap_or_else(|| Rgb24::new_grey(255));
            ViewCell::new()
                .with_character('.')
                .with_bold(true)
                .with_foreground(foreground_colour)
        }
        Tile::Space => ViewCell::new().with_background(Rgb24::new(0, 0, 31)),
        Tile::Window => ViewCell::new()
            .with_character('=')
            .with_foreground(Rgb24::new(255, 255, 255))
            .with_background(Rgb24::new(127, 127, 127)),
        Tile::DoorClosed => ViewCell::new()
            .with_character('+')
            .with_foreground(Rgb24::new(255, 255, 255))
            .with_background(Rgb24::new(127, 127, 127)),
        Tile::DoorOpen => ViewCell::new()
            .with_character('-')
            .with_foreground(Rgb24::new(255, 255, 255))
            .with_background(Rgb24::new(127, 127, 127)),
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
    if to_render_entity.blood {
        view_cell.style.foreground = Some(Rgb24::new(255, 0, 0));
    }
    if let Some(foreground) = view_cell.style.foreground.as_mut() {
        *foreground = foreground.normalised_mul(light_colour);
    }
    if let Some(background) = view_cell.style.background.as_mut() {
        *background = background.normalised_mul(light_colour);
    }
    frame.set_cell_relative(screen_coord.0, depth, view_cell, context);
}

#[derive(Serialize, Deserialize)]
pub struct GameInstance {
    rng: Isaac64Rng,
    game: Game,
    screen_shake: Option<ScreenShake>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum GameStatus {
    Playing,
    Over,
}

pub struct GameToRender<'a> {
    game: &'a Game,
    screen_shake: Option<ScreenShake>,
    status: GameStatus,
}

#[derive(Clone, Copy, Debug)]
pub enum RngSeed {
    Random,
    U64(u64),
}

impl GameInstance {
    fn new(game_config: &GameConfig, mut rng: Isaac64Rng) -> Self {
        Self {
            game: Game::new(game_config, &mut rng),
            rng,
            screen_shake: None,
        }
    }
    pub fn to_render(&self) -> GameToRender {
        GameToRender {
            game: &self.game,
            screen_shake: self.screen_shake,
            status: GameStatus::Playing,
        }
    }
    fn to_render_game_over(&self) -> GameToRender {
        GameToRender {
            game: &self.game,
            screen_shake: self.screen_shake,
            status: GameStatus::Over,
        }
    }
}

pub struct GameData<S: Storage, A: AudioPlayer> {
    instance: Option<GameInstance>,
    controls: Controls,
    rng_seed_source: RngSeedSource,
    last_aim_with_mouse: bool,
    storage_wrapper: StorageWrapper<S>,
    audio_player: A,
    audio_table: AudioTable<A>,
    game_config: GameConfig,
    frontend: Frontend,
}

struct StorageWrapper<S: Storage> {
    storage: S,
    save_key: String,
}

impl<S: Storage> StorageWrapper<S> {
    pub fn save_instance(&mut self, instance: &GameInstance) {
        self.storage
            .store(&self.save_key, instance, STORAGE_FORMAT)
            .expect("failed to save instance");
    }
    pub fn clear_instance(&mut self) {
        let _ = self.storage.remove(&self.save_key);
    }
}

struct RngSeedSource {
    rng: Isaac64Rng,
    next: u64,
}

impl RngSeedSource {
    fn new(rng_seed: RngSeed) -> Self {
        let mut rng = Isaac64Rng::from_entropy();
        let next = match rng_seed {
            RngSeed::Random => rng.gen(),
            RngSeed::U64(seed) => seed,
        };
        Self { rng, next }
    }
    fn next_seed(&mut self) -> u64 {
        let seed = self.next;
        self.next = self.rng.gen();
        seed
    }
}

impl<S: Storage, A: AudioPlayer> GameData<S, A> {
    pub fn new(
        game_config: GameConfig,
        controls: Controls,
        storage: S,
        save_key: String,
        audio_player: A,
        rng_seed: RngSeed,
        frontend: Frontend,
    ) -> Self {
        let mut instance: Option<GameInstance> = match storage.load(&save_key, STORAGE_FORMAT) {
            Ok(instance) => Some(instance),
            Err(e) => {
                log::info!("no instance found: {:?}", e);
                None
            }
        };
        if let Some(instance) = instance.as_mut() {
            instance.game.update_visibility(&game_config);
        }
        let rng_seed_source = RngSeedSource::new(rng_seed);
        let storage_wrapper = StorageWrapper { storage, save_key };
        Self {
            instance,
            controls,
            rng_seed_source,
            last_aim_with_mouse: false,
            storage_wrapper,
            audio_table: AudioTable::new(&audio_player),
            audio_player,
            game_config,
            frontend,
        }
    }
    pub fn has_instance(&self) -> bool {
        self.instance.is_some()
    }
    pub fn instantiate(&mut self) {
        let seed = self.rng_seed_source.next_seed();
        self.frontend.log_rng_seed(seed);
        let rng = Isaac64Rng::seed_from_u64(seed);
        self.instance = Some(GameInstance::new(&self.game_config, rng));
    }
    pub fn save_instance(&mut self) {
        log::info!("saving game...");
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
    pub fn instance(&self) -> Option<&GameInstance> {
        self.instance.as_ref()
    }
    pub fn initial_aim_coord(&self, screen_coord_of_mouse: ScreenCoord) -> Result<ScreenCoord, NoGameInstance> {
        if let Some(instance) = self.instance.as_ref() {
            if self.last_aim_with_mouse {
                Ok(screen_coord_of_mouse)
            } else {
                let player_coord = GameCoord::of_player(instance.game.player_info());
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
        let audio_player = &data.audio_player;
        let audio_table = &data.audio_table;
        let game_config = &data.game_config;
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
                                    AppInput::Wait => Aim::Ignore,
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
                        let game_control_flow = instance.game.handle_tick(since_last, game_config);
                        assert!(game_control_flow.is_none(), "meaningful event while aiming");
                        let mut event_context = EffectContext {
                            rng: &mut instance.rng,
                            screen_shake: &mut instance.screen_shake,
                            audio_player,
                            audio_table,
                            player_coord: GameCoord::of_player(instance.game.player_info()),
                        };
                        event_context.next_frame();
                        for event in instance.game.events() {
                            event_context.handle_event(event);
                        }
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
            view.view(instance.to_render(), context, frame);
            let player_coord = GameCoord::of_player(instance.game.player_info());
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
    GameOver,
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
        let audio_player = &data.audio_player;
        let audio_table = &data.audio_table;
        let game_config = &data.game_config;
        if let Some(instance) = data.instance.as_mut() {
            let player_coord = GameCoord::of_player(instance.game.player_info());
            for injected_input in self.injected_inputs.drain(..) {
                match injected_input {
                    InjectedInput::Fire(screen_coord) => {
                        let GameCoord(raw_game_coord) = ScreenCoordToGameCoord {
                            screen_coord,
                            player_coord,
                        }
                        .compute();
                        if let Some(game_control_flow) =
                            instance.game.handle_input(GameInput::Fire(raw_game_coord), game_config)
                        {
                            match game_control_flow {
                                GameControlFlow::GameOver => return Handled::Return(GameReturn::GameOver),
                            }
                        }
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
                                    let game_control_flow = match app_input {
                                        AppInput::Move(direction) => {
                                            instance.game.handle_input(GameInput::Walk(direction), game_config)
                                        }

                                        AppInput::Aim => return Handled::Return(GameReturn::Aim),
                                        AppInput::Wait => instance.game.handle_input(GameInput::Wait, game_config),
                                    };
                                    if let Some(game_control_flow) = game_control_flow {
                                        match game_control_flow {
                                            GameControlFlow::GameOver => return Handled::Return(GameReturn::GameOver),
                                        }
                                    }
                                }
                            }
                        }
                        Input::Mouse(_) => (),
                    }
                    Handled::Continue(s)
                }
                CommonEvent::Frame(period) => {
                    let maybe_control_flow = instance.game.handle_tick(period, game_config);
                    let mut event_context = EffectContext {
                        rng: &mut instance.rng,
                        screen_shake: &mut instance.screen_shake,
                        audio_player,
                        audio_table,
                        player_coord,
                    };
                    event_context.next_frame();
                    for event in instance.game.events() {
                        event_context.handle_event(event);
                    }
                    if let Some(game_control_flow) = maybe_control_flow {
                        match game_control_flow {
                            GameControlFlow::GameOver => return Handled::Return(GameReturn::GameOver),
                        }
                    }
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
            view.view(instance.to_render(), context, frame);
        }
    }
}

pub struct GameOverEventRoutine<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
    duration: Duration,
}

impl<S: Storage, A: AudioPlayer> GameOverEventRoutine<S, A> {
    pub fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
            duration: Duration::from_millis(0),
        }
    }
}

impl<S: Storage, A: AudioPlayer> EventRoutine for GameOverEventRoutine<S, A> {
    type Return = ();
    type Data = GameData<S, A>;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        let game_config = &data.game_config;
        let audio_player = &data.audio_player;
        let audio_table = &data.audio_table;
        if let Some(instance) = data.instance.as_mut() {
            event_or_peek_with_handled(event_or_peek, self, |mut s, event| match event {
                CommonEvent::Input(input) => match input {
                    Input::Keyboard(_) => Handled::Return(()),
                    Input::Mouse(_) => Handled::Continue(s),
                },
                CommonEvent::Frame(period) => {
                    s.duration += period;
                    const NPC_TURN_PERIOD: Duration = Duration::from_millis(250);
                    if s.duration > NPC_TURN_PERIOD {
                        s.duration -= NPC_TURN_PERIOD;
                        instance.game.handle_npc_turn();
                    }
                    let _ = instance.game.handle_tick(period, game_config);
                    let mut event_context = EffectContext {
                        rng: &mut instance.rng,
                        screen_shake: &mut instance.screen_shake,
                        audio_player,
                        audio_table,
                        player_coord: GameCoord::of_player(instance.game.player_info()),
                    };
                    event_context.next_frame();
                    for event in instance.game.events() {
                        event_context.handle_event(event);
                    }
                    Handled::Continue(s)
                }
            })
        } else {
            Handled::Return(())
        }
    }
    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.instance.as_ref() {
            view.view(instance.to_render_game_over(), context, frame);
        }
    }
}
