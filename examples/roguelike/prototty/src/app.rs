use crate::controls::Controls;
use crate::depth;
use crate::game::{
    AimEventRoutine, GameData, GameEventRoutine, GameOverEventRoutine, GameReturn, GameView, InjectedInput, ScreenCoord,
};
pub use crate::game::{GameConfig, Omniscient, RngSeed};
use common_event::*;
use decorator::*;
use event_routine::*;
use maplit::*;
use menu::{fade_spec, FadeMenuEntryView, MenuInstanceChoose};
use prototty::input::*;
use prototty::*;
use prototty_audio::AudioPlayer;
use prototty_storage::Storage;
use render::{ColModifyDefaultForeground, ColModifyMap, Coord, Rgb24, Style};
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum Frontend {
    Wasm,
    Native,
}

#[derive(Clone, Copy)]
enum MainMenuType {
    Init,
    Pause,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum MainMenuEntry {
    NewGame,
    Resume,
    Quit,
    Save,
    SaveQuit,
    Clear,
}

impl MainMenuEntry {
    fn init(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Native => (vec![NewGame, Quit], hashmap!['n' => NewGame, 'q' => Quit]),
            Frontend::Wasm => (vec![NewGame], hashmap!['n' => NewGame]),
        };
        menu::MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: Some(hotkeys),
        }
        .build()
        .unwrap()
    }
    fn pause(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Native => (
                vec![Resume, SaveQuit, NewGame, Clear],
                hashmap!['r' => Resume, 'q' => SaveQuit, 'n' => NewGame, 'c' => Clear],
            ),
            Frontend::Wasm => (
                vec![Resume, Save, NewGame, Clear],
                hashmap!['r' => Resume, 's' => Save, 'n' => NewGame, 'c' => Clear],
            ),
        };
        menu::MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: Some(hotkeys),
        }
        .build()
        .unwrap()
    }
}

impl<'a> From<&'a MainMenuEntry> for &'a str {
    fn from(main_menu_entry: &'a MainMenuEntry) -> Self {
        match main_menu_entry {
            MainMenuEntry::NewGame => "(n) New Game",
            MainMenuEntry::Resume => "(r) Resume",
            MainMenuEntry::Quit => "(q) Quit",
            MainMenuEntry::SaveQuit => "(q) Save and Quit",
            MainMenuEntry::Save => "(s) Save",
            MainMenuEntry::Clear => "(c) Clear",
        }
    }
}

struct AppData<S: Storage, A: AudioPlayer> {
    frontend: Frontend,
    game: GameData<S, A>,
    main_menu: menu::MenuInstanceChooseOrEscape<MainMenuEntry>,
    main_menu_type: MainMenuType,
    last_mouse_coord: Coord,
}

struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<FadeMenuEntryView<MainMenuEntry>>,
}

impl<S: Storage, A: AudioPlayer> AppData<S, A> {
    fn new(
        game_config: GameConfig,
        frontend: Frontend,
        controls: Controls,
        storage: S,
        save_key: String,
        audio_player: A,
        rng_seed: RngSeed,
    ) -> Self {
        Self {
            frontend,
            game: GameData::new(game_config, controls, storage, save_key, audio_player, rng_seed),
            main_menu: MainMenuEntry::init(frontend).into_choose_or_escape(),
            main_menu_type: MainMenuType::Init,
            last_mouse_coord: Coord::new(0, 0),
        }
    }
}

impl AppView {
    fn new() -> Self {
        use fade_spec::*;
        let spec = Spec {
            normal: Style {
                to: To {
                    foreground: Rgb24::new(127, 127, 127),
                    background: Rgb24::new(0, 0, 0),
                    bold: false,
                    underline: false,
                },
                from: From::current(),
                durations: Durations {
                    foreground: Duration::from_millis(127),
                    background: Duration::from_millis(127),
                },
            },
            selected: Style {
                to: To {
                    foreground: Rgb24::new(255, 255, 255),
                    background: Rgb24::new(87, 87, 87),
                    bold: true,
                    underline: false,
                },
                from: From {
                    foreground: FromCol::Rgb24(Rgb24::new(0, 0, 0)),
                    background: FromCol::Rgb24(Rgb24::new(255, 255, 255)),
                },
                durations: Durations {
                    foreground: Duration::from_millis(63),
                    background: Duration::from_millis(127),
                },
            },
        };
        Self {
            game: GameView::new(),
            main_menu: menu::MenuInstanceView::new(FadeMenuEntryView::new(spec)),
        }
    }
}

impl Default for AppView {
    fn default() -> Self {
        Self::new()
    }
}

struct SelectGame<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S: Storage, A: AudioPlayer> SelectGame<S, A> {
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}
impl<S: Storage, A: AudioPlayer> DataSelector for SelectGame<S, A> {
    type DataInput = AppData<S, A>;
    type DataOutput = GameData<S, A>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.game
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.game
    }
}
impl<S: Storage, A: AudioPlayer> ViewSelector for SelectGame<S, A> {
    type ViewInput = AppView;
    type ViewOutput = GameView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.game
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.game
    }
}
impl<S: Storage, A: AudioPlayer> Selector for SelectGame<S, A> {}

struct SelectMainMenu<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S: Storage, A: AudioPlayer> SelectMainMenu<S, A> {
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}
impl<S: Storage, A: AudioPlayer> ViewSelector for SelectMainMenu<S, A> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<FadeMenuEntryView<MainMenuEntry>>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage, A: AudioPlayer> DataSelector for SelectMainMenu<S, A> {
    type DataInput = AppData<S, A>;
    type DataOutput = menu::MenuInstanceChooseOrEscape<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.main_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.main_menu
    }
}
impl<S: Storage, A: AudioPlayer> Selector for SelectMainMenu<S, A> {}

struct DecorateMainMenu<S, A> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S: Storage, A: AudioPlayer> DecorateMainMenu<S, A> {
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}

struct InitMenu<'e, 'v, E: EventRoutine>(EventRoutineView<'e, 'v, E>);
impl<'a, 'e, 'v, S, A, E> View<&'a AppData<S, A>> for InitMenu<'e, 'v, E>
where
    S: Storage,
    A: AudioPlayer,
    E: EventRoutine<View = AppView, Data = AppData<S, A>>,
{
    fn view<F: Frame, C: ColModify>(&mut self, app_data: &'a AppData<S, A>, context: ViewContext<C>, frame: &mut F) {
        text::StringViewSingleLine::new(Style::new().with_foreground(Rgb24::new_grey(255)).with_bold(true)).view(
            "RIP",
            context.add_offset(Coord::new(1, 1)),
            frame,
        );
        self.0.view(app_data, context.add_offset(Coord::new(1, 3)), frame);
    }
}

impl<S: Storage, A: AudioPlayer> Decorate for DecorateMainMenu<S, A> {
    type View = AppView;
    type Data = AppData<S, A>;
    fn view<E, F, C>(
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: Rgb24::new_grey(0),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: &mut event_routine_view,
                    },
                },
            }
            .view(data, context.add_depth(depth::GAME_MAX + 1), frame);
            event_routine_view.view.game.view(
                instance.to_render(),
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255))
                        .compose(ColModifyMap(|col: Rgb24| col.saturating_scalar_mul_div(1, 3))),
                ),
                frame,
            );
        } else {
            AlignView {
                view: InitMenu(event_routine_view),
                alignment: Alignment::centre(),
            }
            .view(&data, context, frame);
        }
    }
}

struct DecorateGame<S, A> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S, A> DecorateGame<S, A>
where
    S: Storage,
    A: AudioPlayer,
{
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}

impl<S: Storage, A: AudioPlayer> Decorate for DecorateGame<S, A> {
    type View = AppView;
    type Data = AppData<S, A>;
    fn view<E, F, C>(
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        event_routine_view.view(data, context, frame);
    }
}

struct Quit;

struct MouseTracker<S: Storage, A: AudioPlayer, E: EventRoutine> {
    s: PhantomData<S>,
    a: PhantomData<A>,
    e: E,
}

impl<S: Storage, A: AudioPlayer, E: EventRoutine> MouseTracker<S, A, E> {
    fn new(e: E) -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
            e,
        }
    }
}

impl<S: Storage, A: AudioPlayer, E: EventRoutine<Data = AppData<S, A>, Event = CommonEvent>> EventRoutine
    for MouseTracker<S, A, E>
{
    type Return = E::Return;
    type View = E::View;
    type Data = AppData<S, A>;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek.with(
            (self, data),
            |(s, data), event| {
                if let CommonEvent::Input(Input::Mouse(MouseInput::MouseMove { coord, .. })) = event {
                    data.last_mouse_coord = coord;
                }
                s.e.handle(data, view, event_routine::Event::new(event))
                    .map_continue(|e| Self {
                        s: PhantomData,
                        a: PhantomData,
                        e,
                    })
            },
            |(s, data)| {
                s.e.handle(data, view, event_routine::Peek::new())
                    .map_continue(|e| Self {
                        s: PhantomData,
                        a: PhantomData,
                        e,
                    })
            },
        )
    }
    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        self.e.view(data, view, context, frame)
    }
}

fn main_menu<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = Result<MainMenuEntry, menu::Escape>, Data = AppData<S, A>, View = AppView, Event = CommonEvent>
{
    SideEffectThen::new(|data: &mut AppData<S, A>, _| {
        if data.game.has_instance() {
            match data.main_menu_type {
                MainMenuType::Init => {
                    data.main_menu = MainMenuEntry::pause(data.frontend).into_choose_or_escape();
                    data.main_menu_type = MainMenuType::Pause;
                }
                MainMenuType::Pause => (),
            }
        } else {
            match data.main_menu_type {
                MainMenuType::Init => (),
                MainMenuType::Pause => {
                    data.main_menu = MainMenuEntry::init(data.frontend).into_choose_or_escape();
                    data.main_menu_type = MainMenuType::Init;
                }
            }
        }
        menu::FadeMenuInstanceRoutine::new()
            .select(SelectMainMenu::new())
            .decorated(DecorateMainMenu::new())
    })
}

fn game<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = GameReturn, Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    GameEventRoutine::new()
        .select(SelectGame::new())
        .decorated(DecorateGame::new())
}

fn game_injecting_inputs<S: Storage, A: AudioPlayer>(
    inputs: Vec<InjectedInput>,
) -> impl EventRoutine<Return = GameReturn, Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    GameEventRoutine::new_injecting_inputs(inputs)
        .select(SelectGame::new())
        .decorated(DecorateGame::new())
}

fn game_over<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    GameOverEventRoutine::new()
        .select(SelectGame::new())
        .decorated(DecorateGame::new())
}

fn aim<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = Option<ScreenCoord>, Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    SideEffectThen::new(|data: &mut AppData<S, A>, view: &AppView| {
        let game_relative_mouse_coord = view
            .game
            .absolute_coord_to_game_relative_screen_coord(data.last_mouse_coord);
        if let Ok(initial_aim_coord) = data.game.initial_aim_coord(game_relative_mouse_coord) {
            Ei::A(
                AimEventRoutine::new(initial_aim_coord)
                    .select(SelectGame::new())
                    .decorated(DecorateGame::new()),
            )
        } else {
            Ei::B(Value::new(None))
        }
    })
}

enum GameLoopBreak {
    GameOver,
    Pause,
}

fn game_loop<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    Ei::A(game())
        .repeat(|game_return| match game_return {
            GameReturn::Pause => Handled::Return(GameLoopBreak::Pause),
            GameReturn::GameOver => Handled::Return(GameLoopBreak::GameOver),
            GameReturn::Aim => Handled::Continue(Ei::B(aim().and_then(|maybe_screen_coord| {
                make_either!(Ei = A | B);
                if let Some(screen_coord) = maybe_screen_coord {
                    Ei::A(game_injecting_inputs(vec![InjectedInput::Fire(screen_coord)]))
                } else {
                    Ei::B(game())
                }
            }))),
        })
        .and_then(|game_loop_break| {
            make_either!(Ei = A | B);
            match game_loop_break {
                GameLoopBreak::Pause => Ei::A(Value::new(())),
                GameLoopBreak::GameOver => Ei::B(game_over().and_then(|()| {
                    SideEffectThen::new(|data: &mut AppData<S, A>, _| {
                        data.game.clear_instance();
                        Value::new(())
                    })
                })),
            }
        })
}

fn main_menu_cycle<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C | D | E | F);
    main_menu().and_then(|entry| match entry {
        Ok(MainMenuEntry::Quit) => Ei::A(Value::new(Some(Quit))),
        Ok(MainMenuEntry::SaveQuit) => Ei::D(SideEffectThen::new(|data: &mut AppData<S, A>, _| {
            data.game.save_instance();
            Value::new(Some(Quit))
        })),
        Ok(MainMenuEntry::Save) => Ei::E(SideEffectThen::new(|data: &mut AppData<S, A>, _| {
            data.game.save_instance();
            Value::new(None)
        })),
        Ok(MainMenuEntry::Clear) => Ei::F(SideEffectThen::new(|data: &mut AppData<S, A>, _| {
            data.game.clear_instance();
            Value::new(None)
        })),
        Ok(MainMenuEntry::Resume) | Err(menu::Escape) => Ei::B(SideEffectThen::new(|data: &mut AppData<S, A>, _| {
            if data.game.has_instance() {
                Either::Left(game_loop().map(|_| None))
            } else {
                Either::Right(Value::new(None))
            }
        })),
        Ok(MainMenuEntry::NewGame) => Ei::C(SideEffectThen::new(|data: &mut AppData<S, A>, _| {
            data.game.instantiate();
            data.main_menu.menu_instance_mut().set_index(0);
            game_loop().map(|_| None)
        })),
    })
}

fn event_routine<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    MouseTracker::new(
        main_menu_cycle()
            .repeat(|maybe_quit| {
                if let Some(Quit) = maybe_quit {
                    Handled::Return(())
                } else {
                    Handled::Continue(main_menu_cycle())
                }
            })
            .return_on_exit(|data| {
                data.game.save_instance();
                ()
            }),
    )
}

pub fn app<S: Storage, A: AudioPlayer>(
    game_config: GameConfig,
    frontend: Frontend,
    controls: Controls,
    storage: S,
    save_key: String,
    audio_player: A,
    rng_seed: RngSeed,
) -> impl app::App {
    let app_data = AppData::new(
        game_config,
        frontend,
        controls,
        storage,
        save_key,
        audio_player,
        rng_seed,
    );
    let app_view = AppView::new();
    event_routine().app_one_shot_ignore_return(app_data, app_view)
}
