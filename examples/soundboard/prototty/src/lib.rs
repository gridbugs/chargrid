use common_event::*;
use event_routine::*;
use prototty::*;
use prototty_audio::{AudioPlayer, AudioProperties};
use render::*;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
enum MenuEntry {
    Sneeze,
    Bark,
    Horn,
}

impl<'a> From<&'a MenuEntry> for &'a str {
    fn from(menu_entry: &'a MenuEntry) -> Self {
        match menu_entry {
            MenuEntry::Sneeze => "Sneeze",
            MenuEntry::Bark => "Bark",
            MenuEntry::Horn => "Horn",
        }
    }
}

impl MenuEntry {
    fn sound_bytes(self) -> &'static [u8] {
        match self {
            Self::Sneeze => include_bytes!("./sneeze.ogg"),
            Self::Bark => include_bytes!("./bark.ogg"),
            Self::Horn => include_bytes!("./horn.ogg"),
        }
    }
    fn play<P: AudioPlayer>(self, player: &P) {
        let sound = player.load_sound(self.sound_bytes());
        player.play(&sound, AudioProperties::default());
    }
}

impl MenuEntry {
    fn all() -> Vec<Self> {
        vec![Self::Sneeze, Self::Bark, Self::Horn]
    }
}

struct AppData<P: AudioPlayer> {
    menu: menu::MenuInstanceChooseOrEscape<MenuEntry>,
    player: P,
}

impl<P: AudioPlayer> AppData<P> {
    fn new(player: P) -> Self {
        Self {
            menu: menu::MenuInstance::new(MenuEntry::all())
                .unwrap()
                .into_choose_or_escape(),
            player,
        }
    }
}

struct AppView {
    menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl Default for AppView {
    fn default() -> Self {
        Self {
            menu: menu::MenuInstanceView::new(menu::MenuEntryStylePair::new(
                Style::new().with_foreground(Rgb24::new_grey(127)),
                Style::new()
                    .with_bold(true)
                    .with_background(Rgb24::new(255, 255, 255))
                    .with_foreground(Rgb24::new(0, 0, 0)),
            )),
        }
    }
}

struct SelectMenu<P: AudioPlayer>(PhantomData<P>);
impl<P: AudioPlayer> SelectMenu<P> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<P: AudioPlayer> DataSelector for SelectMenu<P> {
    type DataInput = AppData<P>;
    type DataOutput = menu::MenuInstanceChooseOrEscape<MenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.menu
    }
}
impl<P: AudioPlayer> ViewSelector for SelectMenu<P> {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.menu
    }
}
impl<P: AudioPlayer> Selector for SelectMenu<P> {}

struct MenuDecorator<P: AudioPlayer>(PhantomData<P>);
impl<P: AudioPlayer> MenuDecorator<P> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<P: AudioPlayer> Decorate for MenuDecorator<P> {
    type View = AppView;
    type Data = AppData<P>;
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
        event_routine_view.view(data, context.add_offset(Coord::new(1, 1)), frame)
    }
}

fn single<P: AudioPlayer>() -> impl EventRoutine<Return = Option<()>, Data = AppData<P>, View = AppView, Event = Input>
{
    make_either!(Ei = A | B);
    menu::MenuInstanceRoutine::new()
        .select(SelectMenu::new())
        .decorated(MenuDecorator::new())
        .and_then(|maybe_entry| match maybe_entry {
            Ok(entry) => Ei::A(SideEffectThen::new(move |data: &mut AppData<P>, _| {
                entry.play(&data.player);
                Value::new(None)
            })),
            Err(menu::Escape) => Ei::B(Value::new(Some(()))),
        })
}

fn event_routine<P: AudioPlayer>(
) -> impl EventRoutine<Return = (), Data = AppData<P>, View = AppView, Event = CommonEvent> {
    single()
        .repeat(|maybe_entry| match maybe_entry {
            None => Handled::Continue(single()),
            Some(()) => Handled::Return(()),
        })
        .convert_input_to_common_event()
        .return_on_exit(|_| ())
}

pub fn app<P: AudioPlayer>(player: P) -> impl app::App {
    event_routine().app_one_shot_ignore_return(AppData::new(player), AppView::default())
}
