use crate::controls::Controls;
pub use crate::game::RngSeed;
use crate::game::{GameData, GameEventRoutine, GameReturn, GameView};
use common_event::*;
use decorator::*;
use event_routine::*;
use hashbrown::HashMap;
use menu::{menu_entry_view, MenuEntryExtraView, MenuEntryViewInfo, MenuInstanceChoose, MenuInstanceExtraSelect};
use prototty::*;
use prototty_storage::Storage;
use render::{ColModifyDefaultForeground, ColModifyMap, Rgb24, Style};
use std::marker::PhantomData;
use std::time::Duration;
use text::StringViewSingleLine;

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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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
    since_epoch: Duration,
}

pub struct AppView {
    game: GameView,
    main_menu: menu::MenuInstanceView<FadeMenuEntryView>,
}

#[derive(Clone, Copy)]
enum MenuEntryStatus {
    Selected,
    Normal,
}

struct MenuEntryChange {
    since_epoch: Duration,
    change_to: MenuEntryStatus,
    init: bool,
}

struct FadeMenuEntryView {
    last_change: HashMap<MainMenuEntry, MenuEntryChange>,
    selected_fade_to_peak: Duration,
    normal: Rgb24,
    selected_peak: Rgb24,
}

impl FadeMenuEntryView {
    fn new() -> Self {
        Self {
            last_change: HashMap::new(),
            selected_fade_to_peak: Duration::from_millis(187),
            normal: Rgb24::new(127, 127, 127),
            selected_peak: Rgb24::new(255, 255, 255),
        }
    }
}

fn duration_ratio_u8(delta: Duration, period: Duration) -> Result<u8, Duration> {
    if let Some(remaining) = delta.checked_sub(period) {
        Err(remaining)
    } else {
        Ok(((delta.as_micros() * 255) / period.as_micros()) as u8)
    }
}

fn interpolate_rgb24(from: Rgb24, to: Rgb24, by: u8) -> Rgb24 {
    fn interpolate_channel(from: u8, to: u8, by: u8) -> u8 {
        let total_delta = to as i16 - from as i16;
        let current_delta = (total_delta * by as i16) / 255;
        (from as i16 + current_delta) as u8
    }
    Rgb24 {
        r: interpolate_channel(from.r, to.r, by),
        g: interpolate_channel(from.g, to.g, by),
        b: interpolate_channel(from.b, to.b, by),
    }
}

impl MenuEntryExtraView<MainMenuEntry> for FadeMenuEntryView {
    type Extra = Duration;
    fn normal<G: Frame, C: ColModify>(
        &mut self,
        entry: &MainMenuEntry,
        since_epoch: &Duration,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        let current = self.last_change.entry(*entry).or_insert_with(|| MenuEntryChange {
            since_epoch: *since_epoch,
            change_to: MenuEntryStatus::Normal,
            init: true,
        });
        match current.change_to {
            MenuEntryStatus::Normal => (),
            MenuEntryStatus::Selected => {
                current.change_to = MenuEntryStatus::Selected;
                current.init = false;
                current.since_epoch = *since_epoch;
            }
        }
        let foreground = self.normal;
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(Style::new().with_foreground(foreground)),
            context,
            frame,
        )
    }
    fn selected<G: Frame, C: ColModify>(
        &mut self,
        entry: &MainMenuEntry,
        since_epoch: &Duration,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        let current = self.last_change.entry(*entry).or_insert_with(|| MenuEntryChange {
            since_epoch: *since_epoch,
            change_to: MenuEntryStatus::Selected,
            init: true,
        });
        match current.change_to {
            MenuEntryStatus::Selected => (),
            MenuEntryStatus::Normal => {
                current.change_to = MenuEntryStatus::Selected;
                current.init = false;
                current.since_epoch = *since_epoch;
            }
        }
        let foreground = if current.init {
            self.selected_peak
        } else {
            if let Some(time_delta) = since_epoch.checked_sub(current.since_epoch) {
                match duration_ratio_u8(time_delta, self.selected_fade_to_peak) {
                    Ok(ratio) => interpolate_rgb24(self.normal, self.selected_peak, ratio),
                    Err(_) => self.selected_peak,
                }
            } else {
                self.selected_peak
            }
        };
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(Style::new().with_bold(true).with_foreground(foreground)),
            context,
            frame,
        )
    }
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
            since_epoch: Duration::from_millis(0),
        }
    }
}

impl AppView {
    pub fn new() -> Self {
        Self {
            game: GameView,
            main_menu: menu::MenuInstanceView::new(FadeMenuEntryView::new()),
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
    type ViewOutput = menu::MenuInstanceView<FadeMenuEntryView>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl<S: Storage> MenuInstanceExtraSelect for SelectMainMenu<S> {
    type DataInput = AppData<S>;
    type Choose = menu::MenuInstanceChooseOrEscape<MainMenuEntry>;
    type Extra = Duration;

    fn choose<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Choose {
        &input.main_menu
    }
    fn choose_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::Choose {
        &mut input.main_menu
    }
    fn extra<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Extra {
        &input.since_epoch
    }
}

struct DecorateMainMenu<S>(PhantomData<S>);

impl<S: Storage> DecorateMainMenu<S> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

struct InitMenu<'a>(&'a mut AppView);
impl<'a, 'b, S: Storage> View<&'a AppData<S>> for InitMenu<'b> {
    fn view<F: Frame, C: ColModify>(&mut self, app_data: &'a AppData<S>, context: ViewContext<C>, frame: &mut F) {
        text::StringViewSingleLine::new(Style::new().with_bold(true)).view(
            "Template Roguelike",
            context.add_offset(Coord::new(1, 1)),
            frame,
        );
        self.0.main_menu.view(
            (app_data.main_menu.menu_instance(), &app_data.since_epoch),
            context.add_offset(Coord::new(1, 3)),
            frame,
        );
    }
}

impl<S: Storage> DecorateView for DecorateMainMenu<S> {
    type View = AppView;
    type Data = AppData<S>;

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        if data.game.has_instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: Rgb24::new_grey(0),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: &mut view.main_menu,
                    },
                },
            }
            .view(
                (data.main_menu.menu_instance(), &data.since_epoch),
                context.add_depth(1),
                frame,
            );
            if let Some(game) = data.game.game() {
                AlignView {
                    alignment: Alignment::centre(),
                    view: &mut view.game,
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
                view: InitMenu(view),
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

impl<S> DecorateView for DecorateGame<S>
where
    S: Storage,
{
    type View = AppView;
    type Data = AppData<S>;

    fn view<G, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut G)
    where
        G: Frame,
        C: ColModify,
    {
        if let Some(game) = data.game.game() {
            AlignView {
                alignment: Alignment::centre(),
                view: &mut view.game,
            }
            .view(game, context, frame);
        }
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
        menu::MenuInstanceExtraRoutine::new(SelectMainMenu::new())
            .convert_input_to_common_event()
            //.select(SelectMainMenu::new())
            .decorated_view(DecorateMainMenu::new())
    })
}

fn game<S: Storage>() -> impl EventRoutine<Return = GameReturn, Data = AppData<S>, View = AppView, Event = CommonEvent>
{
    GameEventRoutine::new()
        .select(SelectGame::new())
        .decorated_view(DecorateGame::new())
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
                Either::Left(game().map(|GameReturn::Pause| None))
            } else {
                Either::Right(Value::new(None))
            }
        })),
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
        .on_event(|data, event| match event {
            CommonEvent::Input(_) => (),
            CommonEvent::Frame(since_previous) => data.since_epoch += *since_previous,
        })
        .return_on_exit(|_| ())
}
