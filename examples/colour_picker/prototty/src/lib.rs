use prototty::event_routine::EventRoutine;
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

fn inner() -> impl event_routine::EventRoutine<Return = Option<()>, Data = AppData, View = AppView> {
    let main_menu = menu::MenuInstanceExtraRoutine::new(SelectMainMenuExtra);
    let colour_menu = menu::MenuInstanceRoutine::new().select(SelectColourMenu);
    main_menu.and_then(|menu_output| match menu_output {
        menu::MenuOutput::Quit => event_routine::Either::A(event_routine::Value::new(Some(()))),
        menu::MenuOutput::Cancel => event_routine::Either::A(event_routine::Value::new(None)),
        menu::MenuOutput::Finalise(choice) => match choice {
            MainMenuChoice::ChooseColour => event_routine::Either::B(colour_menu.map_side_effect(
                |menu_output, data, _view| match menu_output {
                    menu::MenuOutput::Quit => Some(()),
                    menu::MenuOutput::Cancel => None,
                    menu::MenuOutput::Finalise(choice) => {
                        use ColourMenuChoice::*;
                        let colour = match choice {
                            Red => render::rgb24(255, 0, 0),
                            Green => render::rgb24(0, 127, 0),
                            Blue => render::rgb24(0, 63, 255),
                        };
                        data.main_menu_style = menu::MenuEntryStylePair::new(
                            render::Style::new().with_foreground(colour.scalar_div(2)),
                            render::Style::new().with_foreground(colour).with_bold(true),
                        );
                        None
                    }
                },
            )),
            MainMenuChoice::Quit => event_routine::Either::A(event_routine::Value::new(Some(()))),
        },
    })
}

pub fn test() -> impl event_routine::EventRoutine<Return = (), Data = AppData, View = AppView> {
    inner().repeat(|event| match event {
        Some(()) => event_routine::Handled::Return(()),
        None => event_routine::Handled::Continue(inner()),
    })
}

struct SelectColourMenu;
impl event_routine::ViewSelector for SelectColourMenu {
    type ViewInput = AppView;
    type ViewOutput = menu::MenuInstanceView<menu::MenuEntryStylePair>;

    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.colour_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.colour_menu
    }
}
impl event_routine::DataSelector for SelectColourMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstance<ColourMenuChoice>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.colour_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.colour_menu
    }
}
impl event_routine::Selector for SelectColourMenu {}

struct SelectMainMenuExtra;
impl event_routine::ViewSelector for SelectMainMenuExtra {
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
    type Choice = MainMenuChoice;
    type Extra = menu::MenuEntryStylePair;

    fn menu_instance<'a>(&self, input: &'a Self::DataInput) -> &'a menu::MenuInstance<Self::Choice> {
        &input.main_menu
    }
    fn menu_instance_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut menu::MenuInstance<Self::Choice> {
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
    main_menu: menu::MenuInstance<MainMenuChoice>,
    main_menu_style: menu::MenuEntryStylePair,
    colour_menu: menu::MenuInstance<ColourMenuChoice>,
}

impl AppData {
    pub fn new() -> Self {
        let main_menu = menu::MenuInstance::new(MainMenuChoice::all()).unwrap();
        let main_menu_style = menu::MenuEntryStylePair::new(render::Style::new(), render::Style::new().with_bold(true));
        let colour_menu = menu::MenuInstance::new(ColourMenuChoice::all()).unwrap();
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
