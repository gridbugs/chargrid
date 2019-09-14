use common_event::*;
use event_routine::*;
use prototty::input::Input;
use prototty::*;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
enum MainMenuChoice {
    ChooseColour,
    Quit,
}

impl MainMenuChoice {
    fn all() -> Vec<Self> {
        use MainMenuChoice::*;
        vec![ChooseColour, Quit]
    }
}

impl<'a> From<&'a MainMenuChoice> for &'a str {
    fn from(menu_choice: &'a MainMenuChoice) -> Self {
        match menu_choice {
            MainMenuChoice::ChooseColour => "Choose Colour",
            MainMenuChoice::Quit => "Quit",
        }
    }
}

#[derive(Clone, Copy)]
enum ColourMenuChoice {
    Red,
    Green,
    Blue,
}

impl ColourMenuChoice {
    fn all() -> Vec<Self> {
        use ColourMenuChoice::*;
        vec![Red, Green, Blue]
    }
}

impl<'a> From<&'a ColourMenuChoice> for &'a str {
    fn from(menu_choice: &'a ColourMenuChoice) -> Self {
        match menu_choice {
            ColourMenuChoice::Red => "Red",
            ColourMenuChoice::Green => "Green",
            ColourMenuChoice::Blue => "Blue",
        }
    }
}

fn inner() -> impl EventRoutine<Return = Option<()>, Data = AppData, View = AppView, Event = Input> {
    let main_menu = menu::MenuInstanceExtraRoutine::new(SelectMainMenuExtra);
    let colour_menu = menu::MenuInstanceRoutine::new().select(SelectColourMenu);
    main_menu.and_then(|menu_output| match menu_output {
        Err(menu::Cancel::Quit) => Either::Left(Value::new(Some(()))),
        Err(menu::Cancel::Escape) => Either::Left(Value::new(None)),
        Ok(choice) => match choice {
            MainMenuChoice::ChooseColour => Either::Right(colour_menu.and_then(|menu_output| match menu_output {
                Err(menu::Cancel::Quit) => Either::Left(Value::new(Some(()))),
                Err(menu::Cancel::Escape) => Either::Left(Value::new(None)),
                Ok(choice) => Either::Right(SideEffect::new(move |data: &mut AppData, _: &AppView| {
                    use ColourMenuChoice::*;
                    let colour = match choice {
                        Red => render::Rgb24::new(255, 0, 0),
                        Green => render::Rgb24::new(0, 127, 0),
                        Blue => render::Rgb24::new(0, 63, 255),
                    };
                    data.main_menu_style = menu::MenuEntryStylePair::new(
                        render::Style::new().with_foreground(colour.scalar_div(2)),
                        render::Style::new().with_foreground(colour).with_bold(true),
                    );
                    None
                })),
            })),
            MainMenuChoice::Quit => Either::Left(Value::new(Some(()))),
        },
    })
}

pub fn test() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    inner()
        .repeat(|event| match event {
            Some(()) => Handled::Return(()),
            None => Handled::Continue(inner()),
        })
        .convert_input_to_common_event()
}

struct SelectColourMenu;

impl DataSelector for SelectColourMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrCancel<ColourMenuChoice>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.colour_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.colour_menu
    }
}
impl ViewSelector for SelectColourMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.colour_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.colour_menu
    }
}
impl Selector for SelectColourMenu {}

struct SelectMainMenuExtra;
impl ViewSelector for SelectMainMenuExtra {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>;

    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl menu::MenuInstanceExtraSelect for SelectMainMenuExtra {
    type DataInput = AppData;
    type Choose = menu::MenuInstanceChooseOrCancel<MainMenuChoice>;
    type Extra = menu::MenuEntryStylePair;

    fn choose<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Choose {
        &input.main_menu
    }
    fn choose_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::Choose {
        &mut input.main_menu
    }
    fn extra<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Extra {
        &input.main_menu_style
    }
}

struct ChooseMenuEntryStyle<C> {
    choice: PhantomData<C>,
}
impl<C> ChooseMenuEntryStyle<C> {
    fn new() -> Self {
        Self { choice: PhantomData }
    }
}
impl<C> menu::ChooseStyleFromEntryExtra for ChooseMenuEntryStyle<C> {
    type Extra = menu::MenuEntryStylePair;
    type Entry = C;
    fn choose_style_normal(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> render::Style {
        extra.normal
    }
    fn choose_style_selected(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> render::Style {
        extra.selected
    }
}

pub struct AppData {
    main_menu: menu::MenuInstanceChooseOrCancel<MainMenuChoice>,
    main_menu_style: menu::MenuEntryStylePair,
    colour_menu: menu::MenuInstanceChooseOrCancel<ColourMenuChoice>,
}

impl AppData {
    pub fn new() -> Self {
        let main_menu = menu::MenuInstance::new(MainMenuChoice::all())
            .unwrap()
            .into_choose_or_cancel();
        let main_menu_style = menu::MenuEntryStylePair::new(render::Style::new(), render::Style::new().with_bold(true));
        let colour_menu = menu::MenuInstance::new(ColourMenuChoice::all())
            .unwrap()
            .into_choose_or_cancel();
        Self {
            main_menu,
            main_menu_style,
            colour_menu,
        }
    }
}

pub struct AppView {
    main_menu: menu::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>,
    colour_menu: menu::MenuInstanceView<menu::MenuEntryStylePair>,
}

impl AppView {
    pub fn new() -> Self {
        let main_menu = menu::MenuInstanceView::new(ChooseMenuEntryStyle::new());
        let colour_menu = menu::MenuInstanceView::new(menu::MenuEntryStylePair::new(
            render::Style::new(),
            render::Style::new().with_bold(true),
        ));
        Self { main_menu, colour_menu }
    }
}
