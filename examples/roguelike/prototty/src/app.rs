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
        vec![MainMenuEntry::NewGame, MainMenuEntry::Quit]
    }
    fn pause() -> Vec<Self> {
        vec![MainMenuEntry::Resume, MainMenuEntry::NewGame, MainMenuEntry::Quit]
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

struct SelectInitMenu;
impl ViewSelector for SelectInitMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl DataSelector for SelectInitMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceJustChoose<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.init_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.init_menu
    }
}
impl Selector for SelectInitMenu {}

struct SelectPauseMenu;
impl ViewSelector for SelectPauseMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl DataSelector for SelectPauseMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceJustChoose<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.pause_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.pause_menu
    }
}
impl Selector for SelectPauseMenu {}

struct Quit;

fn main_menu() -> impl EventRoutine<Return = MainMenuEntry, Data = AppData, View = AppView, Event = CommonEvent> {
    with_data(|data: &mut AppData| {
        if data.game.has_instance() {
            Either::A(
                menu::MenuInstanceRoutine::new()
                    .convert_input_to_common_event()
                    .select(SelectPauseMenu),
            )
        } else {
            Either::B(
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
    main_menu().and_then_side_effect(|entry, data, _view| match entry {
        MainMenuEntry::Quit => Either3::A(Value::new(Some(Quit))),
        MainMenuEntry::Resume => Either3::B(game().map(|GameReturn::Pause| None)),
        MainMenuEntry::NewGame => {
            data.game.instantiate();
            Either3::C(game().map(|GameReturn::Pause| None))
        }
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
