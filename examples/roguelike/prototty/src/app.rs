use crate::controls::Controls;
use crate::depth;
use crate::frontend::Frontend;
use crate::game::{
    AimEventRoutine, GameData, GameEventRoutine, GameOverEventRoutine, GameReturn, GameView, InjectedInput,
    MapEventRoutine, ScreenCoord,
};
pub use crate::game::{GameConfig, Omniscient, RngSeed};
use common_event::*;
use decorator::*;
use event_routine::*;
use maplit::hashmap;
use menu::{fade_spec, FadeMenuInstanceView, MenuEntryStringFn, MenuInstanceChoose};
use prototty::input::*;
use prototty::*;
use prototty_audio::AudioPlayer;
use prototty_storage::Storage;
use render::{ColModifyDefaultForeground, ColModifyMap, Coord, Rgb24, Style};
use std::marker::PhantomData;

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
    Options,
}

impl MainMenuEntry {
    fn init(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Graphical | Frontend::AnsiTerminal => (
                vec![NewGame, Options, Quit],
                hashmap!['n' => NewGame, 'o' => Options, 'q' => Quit],
            ),
            Frontend::Web => (vec![NewGame, Options], hashmap!['n' => NewGame, 'o' => Options]),
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
            Frontend::Graphical | Frontend::AnsiTerminal => (
                vec![Resume, SaveQuit, NewGame, Options, Clear],
                hashmap!['r' => Resume, 'q' => SaveQuit, 'o' => Options, 'n' => NewGame, 'c' => Clear],
            ),
            Frontend::Web => (
                vec![Resume, Save, NewGame, Options, Clear],
                hashmap!['r' => Resume, 's' => Save, 'o' => Options, 'n' => NewGame, 'c' => Clear],
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

struct AppData<S: Storage, A: AudioPlayer> {
    frontend: Frontend,
    game: GameData<S, A>,
    main_menu: menu::MenuInstanceChooseOrEscape<MainMenuEntry>,
    main_menu_type: MainMenuType,
    options_menu: menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>>,
    last_mouse_coord: Coord,
    env: Box<dyn Env>,
}

struct AppView {
    game: GameView,
    main_menu: FadeMenuInstanceView,
    options_menu: FadeMenuInstanceView,
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
        fullscreen: Option<Fullscreen>,
        env: Box<dyn Env>,
    ) -> Self {
        let mut game_data = GameData::new(
            game_config,
            controls,
            storage,
            save_key,
            audio_player,
            rng_seed,
            frontend,
        );
        if env.fullscreen_supported() {
            let mut config = game_data.config();
            if fullscreen.is_some() {
                config.fullscreen = true;
            }
            env.set_fullscreen_init(config.fullscreen);
            game_data.set_config(config);
        }
        Self {
            options_menu: OptionsMenuEntry::instance(&env),
            frontend,
            game: game_data,
            main_menu: MainMenuEntry::init(frontend).into_choose_or_escape(),
            main_menu_type: MainMenuType::Init,
            last_mouse_coord: Coord::new(0, 0),
            env,
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
            main_menu: FadeMenuInstanceView::new(spec.clone()),
            options_menu: FadeMenuInstanceView::new(spec.clone()),
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
    type ViewOutput = FadeMenuInstanceView;
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

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
enum OrBack<T> {
    Selection(T),
    Back,
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
enum OptionsMenuEntry {
    ToggleMusic,
    ToggleSfx,
    ToggleFullscreen,
}

impl OptionsMenuEntry {
    fn instance(env: &Box<dyn Env>) -> menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>> {
        use OptionsMenuEntry::*;
        use OrBack::*;
        menu::MenuInstanceBuilder {
            items: if env.fullscreen_supported() {
                vec![
                    Selection(ToggleMusic),
                    Selection(ToggleSfx),
                    Selection(ToggleFullscreen),
                    Back,
                ]
            } else {
                vec![Selection(ToggleMusic), Selection(ToggleSfx), Back]
            },
            selected_index: 0,
            hotkeys: Some(hashmap![
                'm' => Selection(ToggleMusic),
                's' => Selection(ToggleSfx),
                'f' => Selection(ToggleFullscreen),
            ]),
        }
        .build()
        .unwrap()
        .into_choose_or_escape()
    }
}

struct SelectOptionsMenu<S: Storage, A: AudioPlayer> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S: Storage, A: AudioPlayer> SelectOptionsMenu<S, A> {
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}
impl<S: Storage, A: AudioPlayer> ViewSelector for SelectOptionsMenu<S, A> {
    type ViewInput = AppView;
    type ViewOutput = FadeMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.options_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.options_menu
    }
}
impl<S: Storage, A: AudioPlayer> DataSelector for SelectOptionsMenu<S, A> {
    type DataInput = AppData<S, A>;
    type DataOutput = menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.options_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.options_menu
    }
}
impl<S: Storage, A: AudioPlayer> Selector for SelectOptionsMenu<S, A> {}

struct DecorateOptionsMenu<S, A> {
    s: PhantomData<S>,
    a: PhantomData<A>,
}
impl<S: Storage, A: AudioPlayer> DecorateOptionsMenu<S, A> {
    fn new() -> Self {
        Self {
            s: PhantomData,
            a: PhantomData,
        }
    }
}
impl<S: Storage, A: AudioPlayer> Decorate for DecorateOptionsMenu<S, A> {
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

fn options_menu<S: Storage, A: AudioPlayer>() -> impl EventRoutine<
    Return = Result<OrBack<OptionsMenuEntry>, menu::Escape>,
    Data = AppData<S, A>,
    View = AppView,
    Event = CommonEvent,
> {
    SideEffectThen::new_with_view(|data: &mut AppData<S, A>, _: &_| {
        let config = data.game.config();
        let fullscreen = data.env.fullscreen();
        let fullscreen_requires_restart = data.env.fullscreen_requires_restart();
        let menu_entry_string = MenuEntryStringFn::new(
            move |entry: &OrBack<OptionsMenuEntry>, _maybe_selected, buf: &mut String| {
                use std::fmt::Write;
                use OptionsMenuEntry::*;
                use OrBack::*;
                match entry {
                    Back => write!(buf, "back").unwrap(),
                    Selection(entry) => match entry {
                        ToggleMusic => {
                            write!(buf, "(m) Music enabled [{}]", if config.music { '*' } else { ' ' }).unwrap()
                        }
                        ToggleSfx => write!(buf, "(s) Sfx enabled [{}]", if config.sfx { '*' } else { ' ' }).unwrap(),
                        ToggleFullscreen => {
                            if fullscreen_requires_restart {
                                write!(buf, "(f) Fullscreen (requires restart) [{}]", if fullscreen { '*' } else { ' ' }).unwrap()
                            } else {
                                write!(buf, "(f) Fullscreen [{}]", if fullscreen { '*' } else { ' ' }).unwrap()
                            }
                        }
                    },
                }
            },
        );
        menu::FadeMenuInstanceRoutine::new(menu_entry_string)
            .select(SelectOptionsMenu::new())
            .decorated(DecorateOptionsMenu::new())
    })
}

fn options_menu_cycle<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    use OptionsMenuEntry::*;
    use OrBack::*;
    Ei::A(options_menu()).repeat(|choice| match choice {
        Ok(Back) | Err(menu::Escape) => Handled::Return(()),
        Ok(Selection(selection)) => Handled::Continue(Ei::B(SideEffectThen::new_with_view(
            move |data: &mut AppData<S, A>, _: &_| {
                let mut config = data.game.config();
                match selection {
                    ToggleMusic => config.music = !config.music,
                    ToggleSfx => config.sfx = !config.sfx,
                    ToggleFullscreen => {
                        data.env.set_fullscreen(!data.env.fullscreen());
                        config.fullscreen = data.env.fullscreen();
                    }
                }
                data.game.set_config(config);
                options_menu()
            },
        ))),
    })
}

#[derive(Clone, Copy)]
pub struct AutoPlay;

fn main_menu<S: Storage, A: AudioPlayer>(
    auto_play: Option<AutoPlay>,
) -> impl EventRoutine<Return = Result<MainMenuEntry, menu::Escape>, Data = AppData<S, A>, View = AppView, Event = CommonEvent>
{
    make_either!(Ei = A | B);
    SideEffectThen::new_with_view(move |data: &mut AppData<S, A>, _: &_| {
        if auto_play.is_some() {
            if data.game.has_instance() {
                Ei::A(Value::new(Ok(MainMenuEntry::Resume)))
            } else {
                Ei::A(Value::new(Ok(MainMenuEntry::NewGame)))
            }
        } else {
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
            Ei::B(
                menu::FadeMenuInstanceRoutine::new(MenuEntryStringFn::new(
                    |entry: &MainMenuEntry, _maybe_selected, buf: &mut String| {
                        use std::fmt::Write;
                        let s = match entry {
                            MainMenuEntry::NewGame => "(n) New Game",
                            MainMenuEntry::Resume => "(r) Resume",
                            MainMenuEntry::Quit => "(q) Quit",
                            MainMenuEntry::SaveQuit => "(q) Save and Quit",
                            MainMenuEntry::Save => "(s) Save",
                            MainMenuEntry::Clear => "(c) Clear",
                            MainMenuEntry::Options => "(o) Options",
                        };
                        write!(buf, "{}", s).unwrap();
                    },
                ))
                .select(SelectMainMenu::new())
                .decorated(DecorateMainMenu::new()),
            )
        }
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
    SideEffectThen::new_with_view(|data: &mut AppData<S, A>, view: &AppView| {
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

fn map<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    SideEffectThen::new_with_view(|data: &mut AppData<S, A>, _: &_| {
        if let Some(instance) = data.game.instance() {
            Ei::A(
                MapEventRoutine::new_centred_on_player(instance)
                    .select(SelectGame::new())
                    .decorated(DecorateGame::new()),
            )
        } else {
            Ei::B(Value::new(()))
        }
    })
}

enum GameLoopBreak {
    GameOver,
    Pause,
}

fn game_loop<S: Storage, A: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C);
    SideEffect::new_with_view(|data: &mut AppData<S, A>, _: &_| data.game.pre_game_loop())
        .then(|| {
            Ei::A(game())
                .repeat(|game_return| match game_return {
                    GameReturn::Pause => Handled::Return(GameLoopBreak::Pause),
                    GameReturn::GameOver => Handled::Return(GameLoopBreak::GameOver),
                    GameReturn::Map => Handled::Continue(Ei::C(map().then(|| game()))),
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
                            SideEffect::new_with_view(|data: &mut AppData<S, A>, _: &_| {
                                data.game.clear_instance();
                            })
                        })),
                    }
                })
        })
        .then(|| SideEffect::new_with_view(|data: &mut AppData<S, A>, _: &_| data.game.post_game_loop()))
}

fn main_menu_cycle<S: Storage, A: AudioPlayer>(
    auto_play: Option<AutoPlay>,
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C | D | E | F | G);
    main_menu(auto_play).and_then(|entry| match entry {
        Ok(MainMenuEntry::Quit) => Ei::A(Value::new(Some(Quit))),
        Ok(MainMenuEntry::SaveQuit) => Ei::D(SideEffect::new_with_view(|data: &mut AppData<S, A>, _: &_| {
            data.game.save_instance();
            Some(Quit)
        })),
        Ok(MainMenuEntry::Save) => Ei::E(SideEffectThen::new_with_view(|data: &mut AppData<S, A>, _: &_| {
            make_either!(Ei = A | B);
            data.game.save_instance();
            if data.game.has_instance() {
                Ei::A(game_loop().map(|_| None))
            } else {
                Ei::B(Value::new(None))
            }
        })),
        Ok(MainMenuEntry::Clear) => Ei::F(SideEffect::new_with_view(|data: &mut AppData<S, A>, _: &_| {
            data.game.clear_instance();
            None
        })),
        Ok(MainMenuEntry::Resume) | Err(menu::Escape) => {
            Ei::B(SideEffectThen::new_with_view(|data: &mut AppData<S, A>, _: &_| {
                make_either!(Ei = A | B);
                if data.game.has_instance() {
                    Ei::A(game_loop().map(|()| None))
                } else {
                    Ei::B(Value::new(None))
                }
            }))
        }
        Ok(MainMenuEntry::NewGame) => Ei::C(SideEffectThen::new_with_view(|data: &mut AppData<S, A>, _: &_| {
            data.game.instantiate();
            data.main_menu.menu_instance_mut().set_index(0);
            game_loop().map(|()| None)
        })),
        Ok(MainMenuEntry::Options) => Ei::G(options_menu_cycle().map(|_| None)),
    })
}

fn event_routine<S: Storage, A: AudioPlayer>(
    initial_auto_play: Option<AutoPlay>,
) -> impl EventRoutine<Return = (), Data = AppData<S, A>, View = AppView, Event = CommonEvent> {
    MouseTracker::new(
        main_menu_cycle(initial_auto_play)
            .repeat(|maybe_quit| {
                if let Some(Quit) = maybe_quit {
                    Handled::Return(())
                } else {
                    Handled::Continue(main_menu_cycle(None))
                }
            })
            .return_on_exit(|data| {
                data.game.save_instance();
                ()
            }),
    )
}

pub trait Env {
    fn fullscreen(&self) -> bool;
    fn fullscreen_requires_restart(&self) -> bool;
    fn fullscreen_supported(&self) -> bool;
    // hack to get around fact that changing fullscreen mid-game on windows crashes
    fn set_fullscreen_init(&self, fullscreen: bool);
    fn set_fullscreen(&self, fullscreen: bool);
}
pub struct EnvNull;
impl Env for EnvNull {
    fn fullscreen(&self) -> bool {
        false
    }
    fn fullscreen_requires_restart(&self) -> bool {
        false
    }
    fn fullscreen_supported(&self) -> bool {
        false
    }
    fn set_fullscreen(&self, _fullscreen: bool) {}
    fn set_fullscreen_init(&self, _fullscreen: bool) {}
}

pub struct Fullscreen;

pub fn app<S: Storage, A: AudioPlayer>(
    game_config: GameConfig,
    frontend: Frontend,
    controls: Controls,
    storage: S,
    save_key: String,
    audio_player: A,
    rng_seed: RngSeed,
    auto_play: Option<AutoPlay>,
    fullscreen: Option<Fullscreen>,
    env: Box<dyn Env>,
) -> impl app::App {
    let app_data = AppData::new(
        game_config,
        frontend,
        controls,
        storage,
        save_key,
        audio_player,
        rng_seed,
        fullscreen,
        env,
    );
    let app_view = AppView::new();
    event_routine(auto_play).app_one_shot_ignore_return(app_data, app_view)
}
