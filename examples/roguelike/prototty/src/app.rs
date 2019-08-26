use crate::controls::Controls;
pub use crate::game::RngSeed;
use crate::game::{AimEventRoutine, GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use decorator::*;
use event_routine::*;
use menu::{fade_spec, FadeMenuEntryView, MenuInstanceChoose};
use prototty::*;
use prototty_storage::Storage;
use render::{ColModifyDefaultForeground, ColModifyMap, Rgb24, Style};
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
    fn init(frontend: Frontend) -> Vec<Self> {
        use MainMenuEntry::*;
        match frontend {
            Frontend::Native => vec![NewGame, Quit],
            Frontend::Wasm => vec![NewGame],
        }
    }
    fn pause(frontend: Frontend) -> Vec<Self> {
        use MainMenuEntry::*;
        match frontend {
            Frontend::Native => vec![Resume, SaveQuit, NewGame, Clear],
            Frontend::Wasm => vec![Resume, Save, NewGame, Clear],
        }
    }
}

impl<'a> From<&'a MainMenuEntry> for &'a str {
    fn from(main_menu_entry: &'a MainMenuEntry) -> Self {
        match main_menu_entry {
            MainMenuEntry::NewGame => "New Game",
            MainMenuEntry::Resume => "Resume",
            MainMenuEntry::Quit => "Quit",
            MainMenuEntry::SaveQuit => "Save and Quit",
            MainMenuEntry::Save => "Save",
            MainMenuEntry::Clear => "Clear",
        }
    }
}

pub struct AppData<S: Storage> {
    frontend: Frontend,
    game: GameData<S>,
    main_menu: menu::MenuInstanceChooseOrEscape<MainMenuEntry>,
    main_menu_type: MainMenuType,
}

pub struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<FadeMenuEntryView<MainMenuEntry>>,
}

impl<S: Storage> AppData<S> {
    pub fn new(frontend: Frontend, controls: Controls, storage: S, save_key: String, rng_seed: RngSeed) -> Self {
        Self {
            frontend,
            game: GameData::new(controls, storage, save_key, rng_seed),
            main_menu: menu::MenuInstance::new(MainMenuEntry::init(frontend))
                .unwrap()
                .into_choose_or_escape(),
            main_menu_type: MainMenuType::Init,
        }
    }
}

impl AppView {
    pub fn new() -> Self {
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
                    foreground: Duration::from_millis(255),
                    background: Duration::from_millis(255),
                },
            },
            selected: Style {
                to: To {
                    foreground: Rgb24::new(255, 0, 0),
                    background: Rgb24::new(87, 0, 0),
                    bold: true,
                    underline: false,
                },
                from: From {
                    foreground: FromCol::Rgb24(Rgb24::new(0, 0, 0)),
                    background: FromCol::Rgb24(Rgb24::new(255, 0, 0)),
                },
                durations: Durations {
                    foreground: Duration::from_millis(127),
                    background: Duration::from_millis(255),
                },
            },
        };
        Self {
            game: GameView::new(),
            main_menu: menu::MenuInstanceView::new(FadeMenuEntryView::new(spec)),
        }
    }
}

struct SelectGame<S: Storage>(PhantomData<S>);
impl<S: Storage> SelectGame<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: Storage> DataSelector for SelectGame<S> {
    type DataInput = AppData<S>;
    type DataOutput = GameData<S>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.game
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.game
    }
}
impl<S: Storage> ViewSelector for SelectGame<S> {
    type ViewInput = AppView;
    type ViewOutput = GameView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.game
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.game
    }
}
impl<S: Storage> Selector for SelectGame<S> {}

struct SelectMainMenu<S: Storage>(PhantomData<S>);
impl<S: Storage> SelectMainMenu<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: Storage> ViewSelector for SelectMainMenu<S> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<FadeMenuEntryView<MainMenuEntry>>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> DataSelector for SelectMainMenu<S> {
    type DataInput = AppData<S>;
    type DataOutput = menu::MenuInstanceChooseOrEscape<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.main_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> Selector for SelectMainMenu<S> {}

struct DecorateMainMenu<S>(PhantomData<S>);

impl<S: Storage> DecorateMainMenu<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

struct InitMenu<'e, 'v, E: EventRoutine>(EventRoutineView<'e, 'v, E>);
impl<'a, 'e, 'v, S, E> View<&'a AppData<S>> for InitMenu<'e, 'v, E>
where
    S: Storage,
    E: EventRoutine<View = AppView, Data = AppData<S>>,
{
    fn view<F: Frame, C: ColModify>(&mut self, app_data: &'a AppData<S>, context: ViewContext<C>, frame: &mut F) {
        text::StringViewSingleLine::new(Style::new().with_bold(true)).view(
            "Template Roguelike",
            context.add_offset(Coord::new(1, 1)),
            frame,
        );
        self.0.view(app_data, context.add_offset(Coord::new(1, 3)), frame);
    }
}

impl<S: Storage> Decorate for DecorateMainMenu<S> {
    type View = AppView;
    type Data = AppData<S>;
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
        if data.game.has_instance() {
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
            .view(data, context.add_depth(1), frame);
            if let Some(game) = data.game.game() {
                AlignView {
                    alignment: Alignment::centre(),
                    view: &mut event_routine_view.view.game,
                }
                .view(
                    game,
                    context.compose_col_modify(
                        ColModifyDefaultForeground(Rgb24::new_grey(255))
                            .compose(ColModifyMap(|col: Rgb24| col.saturating_scalar_mul_div(1, 3))),
                    ),
                    frame,
                );
            }
        } else {
            AlignView {
                view: InitMenu(event_routine_view),
                alignment: Alignment::centre(),
            }
            .view(&data, context, frame);
        }
    }
}

struct DecorateGame<S>(PhantomData<S>);
impl<S> DecorateGame<S>
where
    S: Storage,
{
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Storage> Decorate for DecorateGame<S> {
    type View = AppView;
    type Data = AppData<S>;
    fn view<E, F, C>(data: &Self::Data, event_routine_view: EventRoutineView<E>, context: ViewContext<C>, frame: &mut F)
    where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        AlignView {
            alignment: Alignment::centre(),
            view: event_routine_view,
        }
        .view(data, context, frame);
    }
}

struct Quit;

fn main_menu<S: Storage>(
) -> impl EventRoutine<Return = Result<MainMenuEntry, menu::Escape>, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    SideEffectThen::new(|data: &mut AppData<S>| {
        if data.game.has_instance() {
            match data.main_menu_type {
                MainMenuType::Init => {
                    data.main_menu = menu::MenuInstance::new(MainMenuEntry::pause(data.frontend))
                        .unwrap()
                        .into_choose_or_escape();
                    data.main_menu_type = MainMenuType::Pause;
                }
                MainMenuType::Pause => (),
            }
        } else {
            match data.main_menu_type {
                MainMenuType::Init => (),
                MainMenuType::Pause => {
                    data.main_menu = menu::MenuInstance::new(MainMenuEntry::init(data.frontend))
                        .unwrap()
                        .into_choose_or_escape();
                    data.main_menu_type = MainMenuType::Init;
                }
            }
        }
        menu::FadeMenuInstanceRoutine::new()
            .select(SelectMainMenu::new())
            .decorated(DecorateMainMenu::new())
    })
}

fn game<S: Storage>() -> impl EventRoutine<Return = GameReturn, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    GameEventRoutine::new()
        .select(SelectGame::new())
        .decorated(DecorateGame::new())
}

fn aim<S: Storage>() -> impl EventRoutine<Return = Option<Coord>, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    make_either!(Ei = A | B);
    SideEffectThen::new(|data: &mut AppData<S>| {
        if let Some(game) = data.game.game() {
            Ei::A(
                AimEventRoutine::new(game.player_coord())
                    .select(SelectGame::new())
                    .decorated(DecorateGame::new()),
            )
        } else {
            Ei::B(Value::new(None))
        }
    })
}

fn game_loop<S: Storage>(
) -> impl EventRoutine<Return = GameReturn, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    Ei::A(game()).repeat(|game_return| match game_return {
        GameReturn::Pause => Handled::Return(GameReturn::Pause),
        GameReturn::Aim => Handled::Continue(Ei::B(aim().and_then(|maybe_coord| {
            if let Some(_coord) = maybe_coord {
                game()
            } else {
                game()
            }
        }))),
    })
}

fn main_menu_cycle<S: Storage>(
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C | D | E | F);
    main_menu().and_then(|entry| match entry {
        Ok(MainMenuEntry::Quit) => Ei::A(Value::new(Some(Quit))),
        Ok(MainMenuEntry::SaveQuit) => Ei::D(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(Some(Quit))
        })),
        Ok(MainMenuEntry::Save) => Ei::E(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(None)
        })),
        Ok(MainMenuEntry::Clear) => Ei::F(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.clear_instance();
            Value::new(None)
        })),
        Ok(MainMenuEntry::Resume) | Err(menu::Escape) => Ei::B(SideEffectThen::new(|data: &mut AppData<S>| {
            if data.game.has_instance() {
                Either::Left(game_loop().map(|_| None))
            } else {
                Either::Right(Value::new(None))
            }
        })),
        Ok(MainMenuEntry::NewGame) => Ei::C(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.instantiate();
            data.main_menu.menu_instance_mut().set_index(0);
            game_loop().map(|_| None)
        })),
    })
}

pub fn event_routine<S: Storage>(
) -> impl EventRoutine<Return = (), Data = AppData<S>, View = AppView, Event = CommonEvent> {
    main_menu_cycle()
        .repeat(|maybe_quit| {
            if let Some(Quit) = maybe_quit {
                Handled::Return(())
            } else {
                Handled::Continue(main_menu_cycle())
            }
        })
        .return_on_exit(|_| ())
}
