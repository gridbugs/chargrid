use crate::controls::Controls;
pub use crate::game::RngSeed;
use crate::game::{GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use event_routine::*;
use menu::MenuInstanceChoose;
use prototty::*;
use prototty_storage::Storage;
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

#[derive(Clone, Copy)]
enum MainMenuEntry {
    NewGame,
    Resume,
    Quit,
    Save,
    SaveQuit,
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
            Frontend::Native => vec![Resume, NewGame, SaveQuit],
            Frontend::Wasm => vec![Resume, NewGame, Save],
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
    main_menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl<S: Storage> AppData<S> {
    pub fn new(frontend: Frontend, controls: Controls, storage: S, rng_seed: RngSeed) -> Self {
        Self {
            frontend,
            game: GameData::new(controls, storage, rng_seed),
            main_menu: menu::MenuInstance::new(MainMenuEntry::init(frontend))
                .unwrap()
                .into_choose_or_escape(),
            main_menu_type: MainMenuType::Init,
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
    type DataOutput = menu::MenuInstanceChooseOrEscape<MainMenuEntry>;
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
        Ok(MainMenuEntry::Quit) => Ei::A(Value::new(Some(Quit))),
        Ok(MainMenuEntry::SaveQuit) => Ei::D(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(Some(Quit))
        })),
        Ok(MainMenuEntry::Save) => Ei::E(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.save_instance();
            Value::new(None)
        })),
        Ok(MainMenuEntry::Resume) | Err(menu::Escape) => Ei::B(game().map(|GameReturn::Pause| None)),
        Ok(MainMenuEntry::NewGame) => Ei::C(SideEffectThen::new(|data: &mut AppData<S>| {
            data.game.instantiate();
            data.main_menu.menu_instance_mut().set_index(0);
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
