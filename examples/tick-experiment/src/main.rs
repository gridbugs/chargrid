mod app {
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

    pub type View = p::MenuInstanceView<p::StrMenuEntryView>;

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
            view: &View,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>;
    }

    struct MenuInstanceTickRoutine;

    impl TickRoutine for MenuInstanceTickRoutine {
        type Return = p::MenuOutput<MenuChoice>;
        type Data = p::MenuInstance<MenuChoice>;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &View,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            if let Some(menu_output) = data.tick_with_mouse(inputs, view) {
                Tick::Return(menu_output)
            } else {
                Tick::Continue(self)
            }
        }
    }
    /*
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
            view: &View,
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
    } */
    /*
    /*
    pub enum Either<A, B> {
        A(A),
        B(B),
    }
    impl<A, B> TickRoutine for Either<A, B>
        where
            A: TickRoutine,
            B: TickRoutine,
    {
    } */

    impl TickRoutine for p::MenuInstance<MenuChoice> {
        type Return = (Self, p::MenuOutput<MenuChoice>);
        type State = Self;
        fn tick<I>(mut self, inputs: I, view: &View) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            if let Some(menu_output) = self.tick_with_mouse(inputs, view) {
                Tick::Return((self, menu_output))
            } else {
                Tick::Continue(self)
            }
        }
        fn state(&self) -> &Self::State {
            self
        }
    }

    pub const MENU_VIEW: p::StrMenuEntryView =
        p::StrMenuEntryView::new(p::Style::new(), p::Style::new().with_bold(true));

    pub fn view() -> View {
        p::MenuInstanceView::new(MENU_VIEW)
    }

    pub fn app() -> impl TickRoutine<Return = Return, State = p::MenuInstance<MenuChoice>>
    {
        p::MenuInstance::new(MenuChoice::all())
            .unwrap()
            .repeat(|(menu, menu_output)| match menu_output {
                p::MenuOutput::Quit => Tick::Return(Return::Quit),
                p::MenuOutput::Cancel => Tick::Continue(menu),
                p::MenuOutput::Finalise(choice) => match choice {
                    MenuChoice::Bar => {
                        println!("Bar");
                        Tick::Continue(menu)
                    }
                    MenuChoice::Foo => {
                        println!("Foo");
                        Tick::Continue(menu)
                    }
                    MenuChoice::Quit => Tick::Return(Return::Quit),
                },
            })
    }

    pub struct App<SM> {
        state_machine: SM,
    }

    pub fn app_(
    ) -> App<impl TickRoutine<Return = Return, State = p::MenuInstance<MenuChoice>>> {
        App {
            state_machine: app(),
        }
    }

    pub enum Return {
        Quit,
    }
    */
}
/*
use app::TickRoutine;
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
    let mut app = app::app();
    let mut view = app::view();
    let mut input_buffer = Vec::with_capacity(64);
    loop {
        context.buffer_input(&mut input_buffer);
        app = match app.tick(input_buffer.drain(..), &view) {
            app::Tick::Continue(app) => app,
            app::Tick::Return(app::Return::Quit) => break,
        };
        context.render(&mut view, app.state()).unwrap();
    }
}*/

fn main() {}
