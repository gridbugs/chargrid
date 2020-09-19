use chargrid::*;
use common_event::*;
use event_routine::*;

#[derive(Clone)]
enum MainMenuEntry {
    ChooseColour,
    Quit,
}

#[derive(Clone, Copy)]
enum ColourMenuEntry {
    Red,
    Green,
    Blue,
}

impl ColourMenuEntry {
    fn to_rgb24(self) -> render::Rgb24 {
        match self {
            ColourMenuEntry::Red => render::Rgb24::new(255, 0, 0),
            ColourMenuEntry::Green => render::Rgb24::new(0, 255, 0),
            ColourMenuEntry::Blue => render::Rgb24::new(0, 0, 255),
        }
    }
}

struct AppData {
    current: Option<render::Rgb24>,
    main_menu: menu::MenuInstanceChooseOrCancel<MainMenuEntry>,
    colour_menu: menu::MenuInstanceChooseOrCancel<ColourMenuEntry>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            current: None,
            main_menu: menu::MenuInstance::new(vec![MainMenuEntry::ChooseColour, MainMenuEntry::Quit])
                .unwrap()
                .into_choose_or_cancel(),
            colour_menu: menu::MenuInstance::new(vec![
                ColourMenuEntry::Red,
                ColourMenuEntry::Green,
                ColourMenuEntry::Blue,
            ])
            .unwrap()
            .into_choose_or_cancel(),
        }
    }
}

#[derive(Default)]
struct AppView {
    main_menu: menu::DynamicStyleMenuInstanceView,
    colour_menu: menu::DynamicStyleMenuInstanceView,
}

struct SelectColourMenu;
impl ViewSelector for SelectColourMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::DynamicStyleMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.colour_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.colour_menu
    }
}
impl DataSelector for SelectColourMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrCancel<ColourMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.colour_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.colour_menu
    }
}
impl Selector for SelectColourMenu {}

struct SelectMainMenu;
impl ViewSelector for SelectMainMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::DynamicStyleMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl DataSelector for SelectMainMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrCancel<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.main_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.main_menu
    }
}
impl Selector for SelectMainMenu {}

fn selected_background(current: Option<render::Rgb24>) -> render::Rgb24 {
    if let Some(current) = current {
        current.scalar_div(2)
    } else {
        render::Rgb24::new_grey(127)
    }
}

fn colour_menu(
) -> impl EventRoutine<Return = Result<ColourMenuEntry, menu::Cancel>, Data = AppData, View = AppView, Event = CommonEvent>
{
    SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
        let current = data.current;
        menu::DynamicStyleMenuInstanceRoutine::new(menu::MenuEntryRichStringFn::new(
            move |entry: menu::MenuEntryToRender<ColourMenuEntry>, buf: &mut String| {
                use std::fmt::Write;
                let cursor = if entry.selected { " >" } else { "  " };
                match entry.entry {
                    ColourMenuEntry::Red => {
                        write!(buf, "{} Red", cursor).unwrap();
                    }
                    ColourMenuEntry::Green => {
                        write!(buf, "{} Green", cursor).unwrap();
                    }
                    ColourMenuEntry::Blue => {
                        write!(buf, "{} Blue", cursor).unwrap();
                    }
                };
                let foreground = entry.entry.to_rgb24();
                let (background, bold) = if entry.selected {
                    (selected_background(current), true)
                } else {
                    (render::Rgb24::new_grey(0), false)
                };
                render::Style::default()
                    .with_foreground(foreground)
                    .with_background(background)
                    .with_bold(bold)
            },
        ))
        .select(SelectColourMenu)
        .convert_input_to_common_event()
    })
}

fn main_menu(
) -> impl EventRoutine<Return = Result<MainMenuEntry, menu::Cancel>, Data = AppData, View = AppView, Event = CommonEvent>
{
    SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
        let current = data.current;
        menu::DynamicStyleMenuInstanceRoutine::new(menu::MenuEntryRichStringFn::new(
            move |entry: menu::MenuEntryToRender<MainMenuEntry>, buf: &mut String| {
                use std::fmt::Write;
                let cursor = if entry.selected { " >" } else { "  " };
                match entry.entry {
                    MainMenuEntry::ChooseColour => write!(buf, "{} Choose Colour", cursor).unwrap(),
                    MainMenuEntry::Quit => write!(buf, "{} Quit", cursor).unwrap(),
                }
                let background = if entry.selected {
                    selected_background(current)
                } else {
                    render::Rgb24::new_grey(0)
                };
                let foreground = render::Rgb24::new_grey(255);
                render::Style::default()
                    .with_foreground(foreground)
                    .with_background(background)
            },
        ))
        .select(SelectMainMenu)
        .convert_input_to_common_event()
    })
}

fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    Ei::A(main_menu()).repeat(|choice| match choice {
        Err(_) | Ok(MainMenuEntry::Quit) => Handled::Return(()),
        Ok(MainMenuEntry::ChooseColour) => Handled::Continue(Ei::B(colour_menu().and_then(|choice| {
            SideEffectThen::new_with_view(move |data: &mut AppData, _: &_| {
                if let Ok(colour_choice) = choice {
                    data.current = Some(colour_choice.to_rgb24());
                }
                main_menu()
            })
        }))),
    })
}

pub fn app() -> impl app::App {
    event_routine().app_one_shot_ignore_return(AppData::default(), AppView::default())
}
