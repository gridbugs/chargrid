mod app {
    use prototty as p;
    use std::marker::PhantomData;

    #[derive(Clone, Copy)]
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

    pub enum Tick<R, C> {
        Return(R),
        Continue(C),
    }

    pub trait TickRoutine: Sized {
        type Return;
        type Data;
        type View;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>;

        fn view<F, R>(
            &self,
            data: &Self::Data,
            view: &mut Self::View,
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

        fn select<S>(self, selector: S) -> Select<Self, S>
        where
            S: Selector<DataOutput = Self::Data, ViewOutput = Self::View>,
        {
            Select { t: self, selector }
        }
    }

    struct MenuInstanceRoutine<C, V> {
        choice: PhantomData<C>,
        view: PhantomData<V>,
    }
    impl<C, V> MenuInstanceRoutine<C, V>
    where
        C: Clone,
        for<'a> V: p::View<&'a p::MenuInstance<C>>,
    {
        fn new() -> Self {
            Self {
                choice: PhantomData,
                view: PhantomData,
            }
        }
    }
    impl<C, V> Clone for MenuInstanceRoutine<C, V>
    where
        C: Clone,
        for<'a> V: p::View<&'a p::MenuInstance<C>>,
    {
        fn clone(&self) -> Self {
            Self::new()
        }
    }
    impl<C, V> Copy for MenuInstanceRoutine<C, V>
    where
        C: Clone,
        for<'a> V: p::View<&'a p::MenuInstance<C>>,
    {
    }

    pub trait Selector {
        type ViewInput;
        type ViewOutput;
        type DataInput;
        type DataOutput;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput;
        fn view_mut<'a>(
            &self,
            input: &'a mut Self::ViewInput,
        ) -> &'a mut Self::ViewOutput;
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput;
        fn data_mut<'a>(
            &self,
            input: &'a mut Self::DataInput,
        ) -> &'a mut Self::DataOutput;
    }

    pub struct Select<T, S> {
        t: T,
        selector: S,
    }

    impl<T, S> TickRoutine for Select<T, S>
    where
        T: TickRoutine,
        S: Selector<DataOutput = T::Data, ViewOutput = T::View>,
    {
        type Return = T::Return;
        type Data = S::DataInput;
        type View = S::ViewInput;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            match self.t.tick(
                self.selector.data_mut(data),
                inputs,
                self.selector.view(view),
            ) {
                Tick::Return(r) => Tick::Return(r),
                Tick::Continue(c) => Tick::Continue(Self { t: c, ..self }),
            }
        }

        fn view<F, R>(
            &self,
            data: &Self::Data,
            view: &mut Self::View,
            context: p::ViewContext<R>,
            frame: &mut F,
        ) where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t.view(
                self.selector.data(data),
                self.selector.view_mut(view),
                context,
                frame,
            )
        }
    }

    impl<C, V> TickRoutine for MenuInstanceRoutine<C, V>
    where
        C: Clone,
        for<'a> V: p::View<&'a p::MenuInstance<C>>,
        V: p::MenuIndexFromScreenCoord,
    {
        type Return = p::MenuOutput<C>;
        type Data = p::MenuInstance<C>;
        type View = V;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
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
        fn view<F, R>(
            &self,
            data: &Self::Data,
            view: &mut Self::View,
            context: p::ViewContext<R>,
            frame: &mut F,
        ) where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            view.view(&data, context, frame);
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
        type View = T::View;

        fn tick<I>(
            mut self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            match self.t.tick(data, inputs, view) {
                Tick::Continue(c) => Tick::Continue(Self { t: c, ..self }),
                Tick::Return(r) => match (self.f)(r) {
                    Tick::Continue(c) => Tick::Continue(Self { t: c, ..self }),
                    Tick::Return(r) => Tick::Return(r),
                },
            }
        }

        fn view<G, R>(
            &self,
            data: &Self::Data,
            view: &mut Self::View,
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

    pub const MENU_INSTANCE_VIEW: p::MenuInstanceView<p::StrMenuEntryView> =
        p::MenuInstanceView::new(MENU_VIEW);

    pub fn tick_routine(
    ) -> impl TickRoutine<Return = Return, Data = AppData, View = AppView> {
        let main_menu = MenuInstanceRoutine::<
            MenuChoice,
            p::MenuInstanceView<p::StrMenuEntryView>,
        >::new();
        main_menu
            .repeat(move |menu_output| match menu_output {
                p::MenuOutput::Quit => Tick::Return(Return::Quit),
                p::MenuOutput::Cancel => Tick::Continue(main_menu),
                p::MenuOutput::Finalise(choice) => match choice {
                    MenuChoice::Bar => {
                        println!("Bar");
                        Tick::Continue(main_menu)
                    }
                    MenuChoice::Foo => {
                        println!("Foo");
                        Tick::Continue(main_menu)
                    }
                    MenuChoice::Quit => Tick::Return(Return::Quit),
                },
            })
            .select(SelectMainMenu)
    }

    struct SelectMainMenu;
    impl Selector for SelectMainMenu {
        type ViewInput = AppView;
        type ViewOutput = p::MenuInstanceView<p::StrMenuEntryView>;
        type DataInput = AppData;
        type DataOutput = p::MenuInstance<MenuChoice>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.main_menu
        }
        fn view_mut<'a>(
            &self,
            input: &'a mut Self::ViewInput,
        ) -> &'a mut Self::ViewOutput {
            &mut input.main_menu
        }
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
            &input.main_menu
        }
        fn data_mut<'a>(
            &self,
            input: &'a mut Self::DataInput,
        ) -> &'a mut Self::DataOutput {
            &mut input.main_menu
        }
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

    pub struct AppView {
        main_menu: p::MenuInstanceView<p::StrMenuEntryView>,
    }

    impl AppView {
        pub fn new() -> Self {
            Self {
                main_menu: MENU_INSTANCE_VIEW,
            }
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
    let mut tick_routine = app::tick_routine();
    let mut app_data = app::AppData::new();
    let mut app_view = app::AppView::new();
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
