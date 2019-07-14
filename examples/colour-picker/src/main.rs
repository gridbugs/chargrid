pub mod app {
    use prototty as p;
    use prototty_tick_routine as t;
    use std::marker::PhantomData;
    use t::TickRoutine;

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

    fn inner() -> impl t::TickRoutine<Return = Option<Return>, Data = AppData, View = AppView> {
        let main_menu = p::menu_tick_routine::MenuInstanceExtraRoutine::new(SelectMainMenuExtra);
        let colour_menu = p::menu_tick_routine::MenuInstanceRoutine::new().select(SelectColourMenu);
        main_menu.and_then(|menu_output| match menu_output {
            p::MenuOutput::Quit => t::Either::A(t::Value::new(Some(Return::Quit))),
            p::MenuOutput::Cancel => t::Either::A(t::Value::new(None)),
            p::MenuOutput::Finalise(choice) => match choice {
                MainMenuChoice::ChooseColour => t::Either::B(colour_menu.map_side_effect(
                    |menu_output, data, _view| match menu_output {
                        p::MenuOutput::Quit => Some(Return::Quit),
                        p::MenuOutput::Cancel => None,
                        p::MenuOutput::Finalise(choice) => {
                            use ColourMenuChoice::*;
                            let colour = match choice {
                                Red => p::rgb24(255, 0, 0),
                                Green => p::rgb24(0, 127, 0),
                                Blue => p::rgb24(0, 63, 255),
                            };
                            data.main_menu_style = p::MenuEntryStylePair::new(
                                p::Style::new().with_foreground(colour.scalar_div(2)),
                                p::Style::new().with_foreground(colour).with_bold(true),
                            );
                            None
                        }
                    },
                )),
                MainMenuChoice::Quit => t::Either::A(t::Value::new(Some(Return::Quit))),
            },
        })
    }

    pub fn test() -> impl t::TickRoutine<Return = Return, Data = AppData, View = AppView> {
        inner().repeat(|event| match event {
            Some(Return::Quit) => t::Tick::Return(Return::Quit),
            None => t::Tick::Continue(inner()),
        })
    }

    struct SelectColourMenu;
    impl t::ViewSelector for SelectColourMenu {
        type ViewInput = AppView;
        type ViewOutput = p::MenuInstanceView<p::MenuEntryStylePair>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.colour_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.colour_menu
        }
    }
    impl t::DataSelector for SelectColourMenu {
        type DataInput = AppData;
        type DataOutput = p::MenuInstance<ColourMenuChoice>;
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
            &input.colour_menu
        }
        fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
            &mut input.colour_menu
        }
    }
    impl t::Selector for SelectColourMenu {}

    struct SelectMainMenuExtra;
    impl t::ViewSelector for SelectMainMenuExtra {
        type ViewInput = AppView;
        type ViewOutput = p::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.main_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.main_menu
        }
    }
    impl p::menu_tick_routine::MenuInstanceExtraSelect for SelectMainMenuExtra {
        type DataInput = AppData;
        type Choice = MainMenuChoice;
        type Extra = p::MenuEntryStylePair;

        fn menu_instance<'a>(&self, input: &'a Self::DataInput) -> &'a p::MenuInstance<Self::Choice> {
            &input.main_menu
        }
        fn menu_instance_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut p::MenuInstance<Self::Choice> {
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
    impl<C> p::ChooseStyleFromEntryExtra for ChooseMenuEntryStyle<C> {
        type Extra = p::MenuEntryStylePair;
        type Entry = C;
        fn choose_style_normal(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> p::Style {
            extra.normal
        }
        fn choose_style_selected(&mut self, _entry: &Self::Entry, extra: &Self::Extra) -> p::Style {
            extra.selected
        }
    }

    pub struct AppData {
        main_menu: p::MenuInstance<MainMenuChoice>,
        main_menu_style: p::MenuEntryStylePair,
        colour_menu: p::MenuInstance<ColourMenuChoice>,
    }

    impl AppData {
        pub fn new() -> Self {
            let main_menu = p::MenuInstance::new(MainMenuChoice::all()).unwrap();
            let main_menu_style = p::MenuEntryStylePair::new(p::Style::new(), p::Style::new().with_bold(true));
            let colour_menu = p::MenuInstance::new(ColourMenuChoice::all()).unwrap();
            Self {
                main_menu,
                main_menu_style,
                colour_menu,
            }
        }
    }

    pub struct AppView {
        main_menu: p::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>,
        colour_menu: p::MenuInstanceView<p::MenuEntryStylePair>,
    }

    impl AppView {
        pub fn new() -> Self {
            let main_menu = p::MenuInstanceView::new(ChooseMenuEntryStyle::new());
            let colour_menu = p::MenuInstanceView::new(p::MenuEntryStylePair::new(
                p::Style::new(),
                p::Style::new().with_bold(true),
            ));
            Self { main_menu, colour_menu }
        }
    }

    pub enum Return {
        Quit,
    }
}

use p::Frame;
use prototty as p;
use prototty_glutin as pg;
use prototty_tick_routine as t;
use std::time::Instant;
use t::TickRoutine;

const WINDOW_SIZE_PIXELS: p::Size = p::Size::new_u16(640, 480);

fn main() {
    let mut context = pg::ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(p::Size::new_u16(16, 16))
        .build()
        .unwrap();
    let mut tick_routine = app::test();
    let mut app_data = app::AppData::new();
    let mut app_view = app::AppView::new();
    let mut input_buffer = Vec::with_capacity(64);
    let mut frame_instant = Instant::now();
    loop {
        let duration = frame_instant.elapsed();
        frame_instant = Instant::now();
        context.buffer_input(&mut input_buffer);
        tick_routine = match tick_routine.tick(&mut app_data, input_buffer.drain(..), &app_view, duration) {
            t::Tick::Continue(tick_routine) => tick_routine,
            t::Tick::Return(app::Return::Quit) => break,
        };
        let mut frame = context.frame();
        tick_routine.view(
            &app_data,
            &mut app_view,
            frame.default_context().add_offset(p::Coord::new(1, 1)),
            &mut frame,
        );
        frame.render().unwrap();
    }
}
