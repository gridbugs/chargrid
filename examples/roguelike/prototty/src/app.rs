use crate::controls::Controls;
use crate::game::{GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use event_routine::*;
use prototty::*;

#[derive(Clone, Copy)]
enum MainMenuEntry {
    NewGame,
    Resume,
    Quit,
}

impl MainMenuEntry {
    fn init() -> Vec<Self> {
        use MainMenuEntry::*;
        vec![NewGame, Quit]
    }
    fn pause() -> Vec<Self> {
        use MainMenuEntry::*;
        vec![Resume, NewGame, Quit]
    }
}

impl<'a> From<&'a MainMenuEntry> for &'a str {
    fn from(main_menu_entry: &'a MainMenuEntry) -> Self {
        match main_menu_entry {
            MainMenuEntry::NewGame => "New Game",
            MainMenuEntry::Resume => "Resume",
            MainMenuEntry::Quit => "Quit",
        }
    }
}

pub struct AppData {
    game: GameData,
    init_menu: menu::MenuInstanceJustChoose<MainMenuEntry>,
    pause_menu: menu::MenuInstanceJustChoose<MainMenuEntry>,
}

pub struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl AppData {
    pub fn new(controls: Controls) -> Self {
        Self {
            game: GameData::new(controls),
            init_menu: menu::MenuInstance::new(MainMenuEntry::init())
                .unwrap()
                .into_just_choose(),
            pause_menu: menu::MenuInstance::new(MainMenuEntry::pause())
                .unwrap()
                .into_just_choose(),
        }
    }
}

impl AppView {
    pub fn new() -> Self {
        Self {
            game: GameView,
            main_menu: menu::MenuInstanceView::new(menu::MenuEntryStylePair::new(
                render::Style::new(),
                render::Style::new().with_bold(true).with_underline(true),
            )),
        }
    }
}

struct SelectGame;
impl_selector! {
    SelectGame
        (AppData->game : GameData)
        (AppView->game : GameView)
}

struct SelectInitMenu;
impl_selector! {
    SelectInitMenu
        (AppData->init_menu : menu::MenuInstanceJustChoose<MainMenuEntry>)
        (AppView->main_menu : menu::MenuInstanceView<menu::MenuEntryStylePair>)
}

struct SelectPauseMenu;
impl_selector! {
    SelectPauseMenu
        (AppData->pause_menu : menu::MenuInstanceJustChoose<MainMenuEntry>)
        (AppView->main_menu : menu::MenuInstanceView<menu::MenuEntryStylePair>)
}

make_either!(E3 = A | B | C);
make_either!(E2 = A | B);

struct Quit;

fn main_menu() -> impl EventRoutine<Return = MainMenuEntry, Data = AppData, View = AppView, Event = CommonEvent> {
    SideEffectThen::new(|data: &mut AppData| {
        if data.game.has_instance() {
            E2::A(
                menu::MenuInstanceRoutine::new()
                    .convert_input_to_common_event()
                    .select(SelectPauseMenu),
            )
        } else {
            E2::B(
                menu::MenuInstanceRoutine::new()
                    .convert_input_to_common_event()
                    .select(SelectInitMenu),
            )
        }
    })
}

fn game() -> impl EventRoutine<Return = GameReturn, Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine.select(SelectGame)
}

fn main_menu_cycle() -> impl EventRoutine<Return = Option<Quit>, Data = AppData, View = AppView, Event = CommonEvent> {
    main_menu().and_then(|entry| match entry {
        MainMenuEntry::Quit => E3::A(Value::new(Some(Quit))),
        MainMenuEntry::Resume => E3::B(game().map(|GameReturn::Pause| None)),
        MainMenuEntry::NewGame => E3::C(SideEffectThen::new(|data: &mut AppData| {
            data.game.instantiate();
            game().map(|GameReturn::Pause| None)
        })),
    })
}

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
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
