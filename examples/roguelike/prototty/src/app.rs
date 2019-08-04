use crate::controls::Controls;
use crate::game::{GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use event_routine::*;
use prototty::*;
use prototty_storage::Storage;
use std::marker::PhantomData;

pub enum Frontend {
    Wasm,
    Native,
}

#[derive(Clone, Copy)]
enum MainMenuEntry {
    NewGame,
    Resume,
    Quit,
    Save,
    SaveQuit,
}

impl MainMenuEntry {
    fn init() -> Vec<Self> {
        use MainMenuEntry::*;
        vec![NewGame, Quit]
    }
    fn pause_native() -> Vec<Self> {
        use MainMenuEntry::*;
        vec![Resume, NewGame, SaveQuit]
    }
    fn pause_wasm() -> Vec<Self> {
        use MainMenuEntry::*;
        vec![Resume, NewGame, Save]
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
        }
    }
}

pub struct AppData<S: Storage> {
    frontend: Frontend,
    game: GameData<S>,
    main_menu: menu::MenuInstanceJustChoose<MainMenuEntry>,
}

pub struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl<S: Storage> AppData<S> {
    pub fn new(frontend: Frontend, controls: Controls, storage: S) -> Self {
        Self {
            frontend,
            game: GameData::new(controls, storage),
            main_menu: menu::MenuInstance::new(MainMenuEntry::init())
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
impl<S: Storage> DataSelector for SelectMainMenu<S> {
    type DataInput = AppData<S>;
    type DataOutput = menu::MenuInstanceJustChoose<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.main_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> ViewSelector for SelectMainMenu<S> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> Selector for SelectMainMenu<S> {}

struct Quit;

fn main_menu<S: Storage>(
) -> impl EventRoutine<Return = MainMenuEntry, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    SideEffectThen::new(|data: &mut AppData<S>| {
        if data.game.has_instance() {
            let menu_entries = match data.frontend {
                Frontend::Native => MainMenuEntry::pause_native(),
                Frontend::Wasm => MainMenuEntry::pause_wasm(),
            };
            data.main_menu = menu::MenuInstance::new(menu_entries).unwrap().into_just_choose();
        } else {
            data.main_menu = menu::MenuInstance::new(MainMenuEntry::init())
                .unwrap()
                .into_just_choose();
        }
        menu::MenuInstanceRoutine::new()
            .convert_input_to_common_event()
            .select(SelectMainMenu::new())
    })
}

fn game<S: Storage>() -> impl EventRoutine<Return = GameReturn, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    GameEventRoutine::new().select(SelectGame::new())
}

fn main_menu_cycle<S: Storage>(
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C | D | E);
    main_menu().and_then(|entry| match entry {
        MainMenuEntry::Quit => Ei::A(Value::new(Some(Quit))),
        MainMenuEntry::SaveQuit => Ei::D(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(Some(Quit))
        })),
        MainMenuEntry::Save => Ei::E(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(None)
        })),
        MainMenuEntry::Resume => Ei::B(game().map(|GameReturn::Pause| None)),
        MainMenuEntry::NewGame => Ei::C(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.instantiate();
            game().map(|GameReturn::Pause| None)
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
