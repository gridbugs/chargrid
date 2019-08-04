use crate::controls::Controls;
use crate::game::{GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use event_routine::*;
use prototty::*;
use prototty_storage::Storage;
use std::marker::PhantomData;

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

pub struct AppData<S: Storage> {
    game: GameData<S>,
    init_menu: menu::MenuInstanceJustChoose<MainMenuEntry>,
    pause_menu: menu::MenuInstanceJustChoose<MainMenuEntry>,
}

pub struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl<S: Storage> AppData<S> {
    pub fn new(controls: Controls, storage: S) -> Self {
        Self {
            game: GameData::new(controls, storage),
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

struct SelectInitMenu<S: Storage>(PhantomData<S>);
impl<S: Storage> SelectInitMenu<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: Storage> DataSelector for SelectInitMenu<S> {
    type DataInput = AppData<S>;
    type DataOutput = menu::MenuInstanceJustChoose<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.init_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.init_menu
    }
}
impl<S: Storage> ViewSelector for SelectInitMenu<S> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> Selector for SelectInitMenu<S> {}

struct SelectPauseMenu<S: Storage>(PhantomData<S>);
impl<S: Storage> SelectPauseMenu<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: Storage> DataSelector for SelectPauseMenu<S> {
    type DataInput = AppData<S>;
    type DataOutput = menu::MenuInstanceJustChoose<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.pause_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.pause_menu
    }
}
impl<S: Storage> ViewSelector for SelectPauseMenu<S> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> Selector for SelectPauseMenu<S> {}

make_either!(E3 = A | B | C);
make_either!(E2 = A | B);

struct Quit;

fn main_menu<S: Storage>(
) -> impl EventRoutine<Return = MainMenuEntry, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    SideEffectThen::new(|data: &mut AppData<S>| {
        if data.game.has_instance() {
            E2::A(
                menu::MenuInstanceRoutine::new()
                    .convert_input_to_common_event()
                    .select(SelectPauseMenu::new()),
            )
        } else {
            E2::B(
                menu::MenuInstanceRoutine::new()
                    .convert_input_to_common_event()
                    .select(SelectInitMenu::new()),
            )
        }
    })
}

fn game<S: Storage>() -> impl EventRoutine<Return = GameReturn, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    GameEventRoutine::new().select(SelectGame::new())
}

fn main_menu_cycle<S: Storage>(
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData<S>, View = AppView, Event = CommonEvent> {
    main_menu().and_then(|entry| match entry {
        MainMenuEntry::Quit => E3::A(Value::new(Some(Quit))),
        MainMenuEntry::Resume => E3::B(game().map(|GameReturn::Pause| None)),
        MainMenuEntry::NewGame => E3::C(SideEffectThen::new(|data: &mut AppData<S>| {
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
