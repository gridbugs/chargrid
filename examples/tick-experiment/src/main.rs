mod tick_routine {
    use std::marker::PhantomData;

    pub enum Tick<R, C> {
        Return(R),
        Continue(C),
    }
    pub trait TickRoutine<A>: Sized {
        type Return;
        type State;
        fn tick(self, args: &mut A) -> Tick<Self::Return, Self>;
        fn state(&self) -> &Self::State;
        fn repeat<U, F: FnMut(Self::Return) -> Tick<U, Self>>(
            self,
            f: F,
        ) -> Repeat<A, Self, F> {
            Repeat {
                t: self,
                f,
                a: PhantomData,
            }
        }
    }

    pub struct Repeat<A, T, F> {
        a: PhantomData<A>,
        t: T,
        f: F,
    }
    impl<T, U, F, A> TickRoutine<A> for Repeat<A, T, F>
    where
        T: TickRoutine<A>,
        F: FnMut(T::Return) -> Tick<U, T>,
    {
        type Return = U;
        type State = T::State;
        fn tick(mut self, args: &mut A) -> Tick<Self::Return, Self> {
            match self.t.tick(args) {
                Tick::Continue(c) => Tick::Continue(Repeat { t: c, ..self }),
                Tick::Return(r) => match (self.f)(r) {
                    Tick::Continue(c) => Tick::Continue(Repeat { t: c, ..self }),
                    Tick::Return(r) => Tick::Return(r),
                },
            }
        }
        fn state(&self) -> &Self::State {
            self.t.state()
        }
    }
}

mod app {
    use super::tick_routine::*;
    use prototty as p;

    #[derive(Clone)]
    enum MenuChoice {
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

    pub const MENU_VIEW: p::StrMenuEntryView =
        p::StrMenuEntryView::new(p::Style::new(), p::Style::new().with_bold(true));

    struct MenuTickRoutine<T: Clone> {
        menu_instance: p::MenuInstance<T>,
    }
    impl<'a, T, I, M> TickRoutine<MenuTickArgs<'a, I, M>> for MenuTickRoutine<T>
    where
        T: Clone,
        I: Iterator<Item = p::Input>,
        M: p::MenuIndexFromScreenCoord,
    {
        type Return = (p::MenuOutput<T>, p::MenuInstance<T>);
        type State = p::MenuInstance<T>;
        fn tick(mut self, args: &mut MenuTickArgs<'a, I, M>) -> Tick<Self::Return, Self> {
            let mut inputs = vec![];
            if let Some(menu_output) = self
                .menu_instance
                .tick_with_mouse(inputs, args.menu_index_from_screen_coord)
            {
                Tick::Return((menu_output, self.menu_instance))
            } else {
                Tick::Continue(self)
            }
        }
        fn state(&self) -> &Self::State {
            &self.menu_instance
        }
    }

    pub struct MenuTickArgs<'a, I, M>
    where
        I: Iterator<Item = p::Input>,
        M: p::MenuIndexFromScreenCoord,
    {
        pub inputs: I,
        pub menu_index_from_screen_coord: &'a M,
    }

    pub struct GenApp<T> {
        routine: T,
    }
    pub fn app<'a, I: Iterator<Item = p::Input>, M: 'a + p::MenuIndexFromScreenCoord>(
    ) -> impl TickRoutine<
        MenuTickArgs<'a, I, M>,
        Return = Return,
        State = p::MenuInstance<MenuChoice>,
    > {
        let menu_instance = p::MenuInstance::new(MenuChoice::all()).unwrap();
        let menu_routine = MenuTickRoutine { menu_instance };
        menu_routine.repeat(|(menu_output, menu_instance)| match menu_output {
            p::MenuOutput::Quit => Tick::Return(Return::Quit),
            p::MenuOutput::Cancel => Tick::Continue(MenuTickRoutine { menu_instance }),
            p::MenuOutput::Finalise(choice) => match choice {
                MenuChoice::Quit => Tick::Return(Return::Quit),
                MenuChoice::Foo => {
                    println!("Foo");
                    Tick::Continue(MenuTickRoutine { menu_instance })
                }
                MenuChoice::Bar => {
                    println!("Bar");
                    Tick::Continue(MenuTickRoutine { menu_instance })
                }
            },
        })
    }
    pub struct App {
        menu_instance: p::MenuInstance<MenuChoice>,
    }
    impl App {
        pub fn new() -> Self {
            let menu_instance = p::MenuInstance::new(MenuChoice::all()).unwrap();
            Self { menu_instance }
        }
        pub fn tick<I>(&mut self, inputs: I, view: &AppView) -> Option<Return>
        where
            I: Iterator<Item = p::Input>,
        {
            if let Some(menu_output) = self
                .menu_instance
                .tick_with_mouse(inputs, &view.menu_instance_view)
            {
                match menu_output {
                    p::MenuOutput::Quit => return Some(Return::Quit),
                    p::MenuOutput::Cancel => (),
                    p::MenuOutput::Finalise(choice) => match choice {
                        MenuChoice::Quit => return Some(Return::Quit),
                        MenuChoice::Foo => println!("Foo"),
                        MenuChoice::Bar => println!("Bar"),
                    },
                }
            }
            None
        }
    }

    pub struct AppView {
        pub menu_instance_view: p::MenuInstanceView<p::StrMenuEntryView>,
    }
    impl AppView {
        pub fn new() -> Self {
            let menu_instance_view = p::MenuInstanceView::new(MENU_VIEW);
            Self { menu_instance_view }
        }
    }
    impl<'a> p::View<&'a p::MenuInstance<MenuChoice>> for AppView {
        fn view<G: p::ViewGrid, R: p::ViewTransformRgb24>(
            &mut self,
            menu_instance: &'a p::MenuInstance<MenuChoice>,
            context: p::ViewContext<R>,
            grid: &mut G,
        ) {
            self.menu_instance_view.view(
                menu_instance,
                context.add_offset(p::Coord::new(1, 1)),
                grid,
            );
        }
    }
    pub enum Return {
        Quit,
    }
}

use prototty as p;
use prototty_glutin as pg;
use tick_routine::*;

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
    let mut app_view = app::AppView::new();
    let mut input_buffer = Vec::with_capacity(64);
    loop {
        {
            context.buffer_input(&mut input_buffer);
        }
        app = {
            let mut args = app::MenuTickArgs {
                inputs: input_buffer.drain(..),
                menu_index_from_screen_coord: &app_view.menu_instance_view,
            };
            match app.tick(&mut args) {
                Tick::Continue(app) => app,
                Tick::Return(app::Return::Quit) => break,
            }
        };
        context.render(&mut app_view, app.state()).unwrap();
    }
}
