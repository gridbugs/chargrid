mod app {
    use p::View;
    use prototty as p;

    #[derive(Clone)]
    pub enum MenuChoice {
        Foo,
        Bar,
        Quit,
    }

    impl MenuChoice {
        fn all() -> Vec<Self> {
            use MenuChoice::*;
            vec![Foo, Bar, Quit]
        }
    }

    impl<'a> From<&'a MenuChoice> for &'a str {
        fn from(menu_choice: &'a MenuChoice) -> Self {
            match menu_choice {
                MenuChoice::Bar => "Bar",
                MenuChoice::Foo => "Foo",
                MenuChoice::Quit => "Quit",
            }
        }
    }

    pub type AppView = p::MenuInstanceView<p::StrMenuEntryView>;

    pub enum Tick<R, C> {
        Return(R),
        Continue(C),
    }

    pub trait TickRoutine: Sized {
        type Return;
        type Data;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &AppView,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>;

        fn view<F, R>(
            &self,
            data: &Self::Data,
            view: &mut AppView,
            context: p::ViewContext<R>,
            frame: &mut F,
        ) where
            F: p::Frame,
            R: p::ViewTransformRgb24;

        fn repeat<U, F>(self, f: F) -> Repeat<Self, F>
        where
            F: FnMut(Self::Return) -> Tick<U, Self>,
        {
            Repeat { t: self, f }
        }
    }

    struct MainMenuInstance;

    impl TickRoutine for MainMenuInstance {
        type Return = p::MenuOutput<MenuChoice>;
        type Data = AppData;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &AppView,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            if let Some(menu_output) = data.main_menu.tick_with_mouse(inputs, view) {
                Tick::Return(menu_output)
            } else {
                Tick::Continue(self)
            }
        }

        fn view<F, R>(
            &self,
            data: &Self::Data,
            view: &mut AppView,
            context: p::ViewContext<R>,
            frame: &mut F,
        ) where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            view.view(&data.main_menu, context, frame);
        }
    }

    pub struct Repeat<T, F> {
        t: T,
        f: F,
    }
    impl<T, U, F> TickRoutine for Repeat<T, F>
    where
        T: TickRoutine,
        F: FnMut(T::Return) -> Tick<U, T>,
    {
        type Return = U;
        type Data = T::Data;
        fn tick<I>(
            mut self,
            data: &mut Self::Data,
            inputs: I,
            view: &AppView,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            match self.t.tick(data, inputs, view) {
                Tick::Continue(c) => Tick::Continue(Repeat { t: c, ..self }),
                Tick::Return(r) => match (self.f)(r) {
                    Tick::Continue(c) => Tick::Continue(Repeat { t: c, ..self }),
                    Tick::Return(r) => Tick::Return(r),
                },
            }
        }

        fn view<G, R>(
            &self,
            data: &Self::Data,
            view: &mut AppView,
            context: p::ViewContext<R>,
            frame: &mut G,
        ) where
            G: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t.view(data, view, context, frame)
        }
    }

    pub const MENU_VIEW: p::StrMenuEntryView =
        p::StrMenuEntryView::new(p::Style::new(), p::Style::new().with_bold(true));

    pub fn view() -> AppView {
        p::MenuInstanceView::new(MENU_VIEW)
    }

    pub fn tick_routine() -> impl TickRoutine<Return = Return, Data = AppData> {
        MainMenuInstance.repeat(|menu_output| match menu_output {
            p::MenuOutput::Quit => Tick::Return(Return::Quit),
            p::MenuOutput::Cancel => Tick::Continue(MainMenuInstance),
            p::MenuOutput::Finalise(choice) => match choice {
                MenuChoice::Bar => {
                    println!("Bar");
                    Tick::Continue(MainMenuInstance)
                }
                MenuChoice::Foo => {
                    println!("Foo");
                    Tick::Continue(MainMenuInstance)
                }
                MenuChoice::Quit => Tick::Return(Return::Quit),
            },
        })
    }

    pub struct AppData {
        main_menu: p::MenuInstance<MenuChoice>,
    }

    impl AppData {
        pub fn new() -> Self {
            let main_menu = p::MenuInstance::new(MenuChoice::all()).unwrap();
            Self { main_menu }
        }
    }

    pub enum Return {
        Quit,
    }
}

use app::TickRoutine;
use p::Frame;
use prototty as p;
use prototty_glutin as pg;

const WINDOW_SIZE_PIXELS: p::Size = p::Size::new_u16(640, 480);

fn main() {
    let mut context =
        pg::ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
            .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
            .with_window_dimensions(WINDOW_SIZE_PIXELS)
            .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
            .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
            .with_font_scale(16.0, 16.0)
            .with_cell_dimensions(p::Size::new_u16(16, 16))
            .build()
            .unwrap();
    let mut app_data = app::AppData::new();
    let mut tick_routine = app::tick_routine();
    let mut app_view = app::view();
    let mut input_buffer = Vec::with_capacity(64);
    loop {
        context.buffer_input(&mut input_buffer);
        tick_routine =
            match tick_routine.tick(&mut app_data, input_buffer.drain(..), &app_view) {
                app::Tick::Continue(tick_routine) => tick_routine,
                app::Tick::Return(app::Return::Quit) => break,
            };
        let mut frame = context.frame();
        tick_routine.view(
            &app_data,
            &mut app_view,
            frame.default_context(),
            &mut frame,
        );
        frame.render().unwrap();
    }
}
