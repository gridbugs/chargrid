pub mod app {
    use p::event_routine::EventRoutine;
    use prototty as p;
    use std::marker::PhantomData;

    #[derive(Clone, Copy)]
    pub enum MainMenuChoice {
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
    pub enum ColourMenuChoice {
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

    fn inner() -> impl p::event_routine::EventRoutine<Return = Option<Return>, Data = AppData, View = AppView> {
        let main_menu = p::menu::MenuInstanceExtraRoutine::new(SelectMainMenuExtra);
        let colour_menu = p::menu::MenuInstanceRoutine::new().select(SelectColourMenu);
        main_menu.and_then(|menu_output| match menu_output {
            p::menu::MenuOutput::Quit => p::event_routine::Either::A(p::event_routine::Value::new(Some(Return::Quit))),
            p::menu::MenuOutput::Cancel => p::event_routine::Either::A(p::event_routine::Value::new(None)),
            p::menu::MenuOutput::Finalise(choice) => match choice {
                MainMenuChoice::ChooseColour => p::event_routine::Either::B(colour_menu.map_side_effect(
                    |menu_output, data, _view| match menu_output {
                        p::menu::MenuOutput::Quit => Some(Return::Quit),
                        p::menu::MenuOutput::Cancel => None,
                        p::menu::MenuOutput::Finalise(choice) => {
                            use ColourMenuChoice::*;
                            let colour = match choice {
                                Red => p::render::rgb24(255, 0, 0),
                                Green => p::render::rgb24(0, 127, 0),
                                Blue => p::render::rgb24(0, 63, 255),
                            };
                            data.main_menu_style = p::menu::MenuEntryStylePair::new(
                                p::render::Style::new().with_foreground(colour.scalar_div(2)),
                                p::render::Style::new().with_foreground(colour).with_bold(true),
                            );
                            None
                        }
                    },
                )),
                MainMenuChoice::Quit => p::event_routine::Either::A(p::event_routine::Value::new(Some(Return::Quit))),
            },
        })
    }

    pub fn test() -> impl p::event_routine::EventRoutine<Return = Return, Data = AppData, View = AppView> {
        inner().repeat(|event| match event {
            Some(Return::Quit) => p::event_routine::Handled::Return(Return::Quit),
            None => p::event_routine::Handled::Continue(inner()),
        })
    }

    struct SelectColourMenu;
    impl p::event_routine::ViewSelector for SelectColourMenu {
        type ViewInput = AppView;
        type ViewOutput = p::menu::MenuInstanceView<p::menu::MenuEntryStylePair>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.colour_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.colour_menu
        }
    }
    impl p::event_routine::DataSelector for SelectColourMenu {
        type DataInput = AppData;
        type DataOutput = p::menu::MenuInstance<ColourMenuChoice>;
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
            &input.colour_menu
        }
        fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
            &mut input.colour_menu
        }
    }
    impl p::event_routine::Selector for SelectColourMenu {}

    struct SelectMainMenuExtra;
    impl p::event_routine::ViewSelector for SelectMainMenuExtra {
        type ViewInput = AppView;
        type ViewOutput = p::menu::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.main_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.main_menu
        }
    }
    impl p::menu::MenuInstanceExtraSelect for SelectMainMenuExtra {
        type DataInput = AppData;
        type Choice = MainMenuChoice;
        type Extra = p::menu::MenuEntryStylePair;

        fn menu_instance<'a>(&self, input: &'a Self::DataInput) -> &'a p::menu::MenuInstance<Self::Choice> {
            &input.main_menu
        }
        fn menu_instance_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut p::menu::MenuInstance<Self::Choice> {
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
    impl<C> p::menu::ChooseStyleFromEntryExtra for ChooseMenuEntryStyle<C> {
        type Extra = p::menu::MenuEntryStylePair;
        type Entry = C;
        fn choose_style_normal(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> p::render::Style {
            extra.normal
        }
        fn choose_style_selected(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> p::render::Style {
            extra.selected
        }
    }

    pub struct AppData {
        main_menu: p::menu::MenuInstance<MainMenuChoice>,
        main_menu_style: p::menu::MenuEntryStylePair,
        colour_menu: p::menu::MenuInstance<ColourMenuChoice>,
    }

    impl AppData {
        pub fn new() -> Self {
            let main_menu = p::menu::MenuInstance::new(MainMenuChoice::all()).unwrap();
            let main_menu_style =
                p::menu::MenuEntryStylePair::new(p::render::Style::new(), p::render::Style::new().with_bold(true));
            let colour_menu = p::menu::MenuInstance::new(ColourMenuChoice::all()).unwrap();
            Self {
                main_menu,
                main_menu_style,
                colour_menu,
            }
        }
    }

    pub struct AppView {
        main_menu: p::menu::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>,
        colour_menu: p::menu::MenuInstanceView<p::menu::MenuEntryStylePair>,
    }

    impl AppView {
        pub fn new() -> Self {
            let main_menu = p::menu::MenuInstanceView::new(ChooseMenuEntryStyle::new());
            let colour_menu = p::menu::MenuInstanceView::new(p::menu::MenuEntryStylePair::new(
                p::render::Style::new(),
                p::render::Style::new().with_bold(true),
            ));
            Self { main_menu, colour_menu }
        }
    }

    pub enum Return {
        Quit,
    }
}

use p::event_routine::{event, EventRoutine};
use p::render::*;
use prototty as p;
use prototty_glutin as pg;
use std::time::Instant;

const WINDOW_SIZE_PIXELS: p::render::Size = p::render::Size::new_u16(640, 480);

fn main() {
    let mut context = pg::ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(p::render::Size::new_u16(16, 16))
        .build()
        .unwrap();
    let mut event_routine = app::test();
    let mut app_data = app::AppData::new();
    let mut app_view = app::AppView::new();
    let mut frame_instant = Instant::now();
    loop {
        let duration = frame_instant.elapsed();
        frame_instant = Instant::now();
        event_routine = {
            let mut maybe_event_routine = Some(event_routine);
            context.poll_input(|input| {
                maybe_event_routine = if let Some(event_routine) = maybe_event_routine.take() {
                    match event_routine.handle_event(&mut app_data, &app_view, event::Input(input)) {
                        p::event_routine::Handled::Continue(event_routine) => Some(event_routine),
                        p::event_routine::Handled::Return(app::Return::Quit) => None,
                    }
                } else {
                    None
                };
            });
            if let Some(event_routine) = maybe_event_routine {
                event_routine
            } else {
                break;
            }
        };
        event_routine = match event_routine.handle_event(&mut app_data, &app_view, event::Frame(duration)) {
            p::event_routine::Handled::Continue(event_routine) => event_routine,
            p::event_routine::Handled::Return(app::Return::Quit) => break,
        };
        let mut frame = context.frame();
        event_routine.view(
            &app_data,
            &mut app_view,
            frame.default_context().add_offset(p::render::Coord::new(1, 1)),
            &mut frame,
        );
        frame.render().unwrap();
    }
}
