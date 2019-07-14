pub mod app {
    use p::Tick;
    use prototty as p;
    use std::marker::PhantomData;
    use std::time::Duration;

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

    pub trait TickRoutine: Sized {
        type Return;
        type Data;
        type View;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>;

        fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24;

        fn peek(self, _data: &mut Self::Data) -> Tick<Self::Return, Self> {
            Tick::Continue(self)
        }

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

        fn and_then<U, F>(self, f: F) -> AndThen<Self, U, F>
        where
            U: TickRoutine<Data = Self::Data, View = Self::View>,
            F: FnOnce(Self::Return) -> U,
        {
            AndThen::First { t: self, f }
        }

        fn map<F, U>(self, f: F) -> Map<Self, F>
        where
            F: FnOnce(Self::Return) -> U,
        {
            Map { t: self, f }
        }

        fn map_side_effect<F, U>(self, f: F) -> MapSideEffect<Self, F>
        where
            F: FnOnce(Self::Return, &mut Self::Data, &Self::View) -> U,
        {
            MapSideEffect { t: self, f }
        }
    }

    pub struct Value<T, D, V> {
        value: T,
        data: PhantomData<D>,
        view: PhantomData<V>,
    }

    impl<T, D, V> Value<T, D, V> {
        pub fn new(value: T) -> Self {
            Self {
                value,
                data: PhantomData,
                view: PhantomData,
            }
        }
    }

    impl<T, D, V> TickRoutine for Value<T, D, V> {
        type Return = T;
        type Data = D;
        type View = V;
        fn tick<I>(
            self,
            data: &mut Self::Data,
            _inputs: I,
            _view: &Self::View,
            _duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            self.peek(data)
        }

        fn view<F, R>(&self, _data: &Self::Data, _view: &mut Self::View, _context: p::ViewContext<R>, _frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
        }

        fn peek(self, _data: &mut Self::Data) -> Tick<Self::Return, Self> {
            Tick::Return(self.value)
        }
    }

    pub struct MapSideEffect<T, F> {
        t: T,
        f: F,
    }
    impl<T, U, F> TickRoutine for MapSideEffect<T, F>
    where
        T: TickRoutine,
        F: FnOnce(T::Return, &mut T::Data, &T::View) -> U,
    {
        type Return = U;
        type Data = T::Data;
        type View = T::View;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            let Self { t, f } = self;
            match t.tick(data, inputs, view, duration) {
                Tick::Continue(t) => Tick::Continue(Self { t, f }),
                Tick::Return(r) => Tick::Return(f(r, data, view)),
            }
        }

        fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut G)
        where
            G: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t.view(data, view, context, frame)
        }
    }
    pub struct Map<T, F> {
        t: T,
        f: F,
    }

    impl<T, U, F> TickRoutine for Map<T, F>
    where
        T: TickRoutine,
        F: FnOnce(T::Return) -> U,
    {
        type Return = U;
        type Data = T::Data;
        type View = T::View;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            let Self { t, f } = self;
            match t.tick(data, inputs, view, duration) {
                Tick::Continue(t) => Tick::Continue(Self { t, f }),
                Tick::Return(r) => Tick::Return(f(r)),
            }
        }

        fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut G)
        where
            G: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t.view(data, view, context, frame)
        }
    }

    pub enum AndThen<T, U, F> {
        First { t: T, f: F },
        Second(U),
    }

    impl<T, U, F> TickRoutine for AndThen<T, U, F>
    where
        T: TickRoutine,
        U: TickRoutine<Data = T::Data, View = T::View>,
        F: FnOnce(T::Return) -> U,
    {
        type Return = U::Return;
        type Data = T::Data;
        type View = T::View;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            match self {
                AndThen::First { t, f } => match t.tick(data, inputs, view, duration) {
                    Tick::Continue(t) => Tick::Continue(AndThen::First { t, f }),
                    Tick::Return(r) => f(r).peek(data).map_continue(AndThen::Second),
                },
                AndThen::Second(u) => u.tick(data, inputs, view, duration).map_continue(AndThen::Second),
            }
        }

        fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut G)
        where
            G: p::Frame,
            R: p::ViewTransformRgb24,
        {
            match self {
                AndThen::First { ref t, .. } => t.view(data, view, context, frame),
                AndThen::Second(ref u) => u.view(data, view, context, frame),
            }
        }
    }

    pub enum Either<A, B> {
        A(A),
        B(B),
    }

    impl<A, B> TickRoutine for Either<A, B>
    where
        A: TickRoutine,
        B: TickRoutine<Data = A::Data, View = A::View, Return = A::Return>,
    {
        type Return = A::Return;
        type Data = A::Data;
        type View = A::View;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            match self {
                Either::A(a) => a.tick(data, inputs, view, duration).map_continue(Either::A),
                Either::B(b) => b.tick(data, inputs, view, duration).map_continue(Either::B),
            }
        }

        fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            match self {
                Either::A(a) => a.view(data, view, context, frame),
                Either::B(b) => b.view(data, view, context, frame),
            }
        }
    }

    pub trait DataSelector {
        type DataInput;
        type DataOutput;
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput;
        fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput;
    }

    pub trait ViewSelector {
        type ViewInput;
        type ViewOutput;
        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput;
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput;
    }

    pub trait Selector: DataSelector + ViewSelector {}

    #[derive(Clone, Copy)]
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
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            let Self { t, selector } = self;
            t.tick(selector.data_mut(data), inputs, selector.view(view), duration)
                .map_continue(|t| Self { t, selector })
        }

        fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t
                .view(self.selector.data(data), self.selector.view_mut(view), context, frame)
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
            _duration: Duration,
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
        fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            view.view(&data, context, frame);
        }
    }

    trait MenuInstanceExtraSelect {
        type DataInput;
        type Choice: Clone;
        type Extra;
        fn menu_instance<'a>(&self, input: &'a Self::DataInput) -> &'a p::MenuInstance<Self::Choice>;
        fn menu_instance_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut p::MenuInstance<Self::Choice>;
        fn extra<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Extra;
    }

    struct MenuInstanceExtraRoutine<S> {
        s: S,
    }
    impl<S> MenuInstanceExtraRoutine<S>
    where
        S: MenuInstanceExtraSelect,
    {
        fn new(s: S) -> Self {
            Self { s }
        }
    }

    impl<S> TickRoutine for MenuInstanceExtraRoutine<S>
    where
        S: MenuInstanceExtraSelect + ViewSelector,
        S::ViewOutput: p::MenuIndexFromScreenCoord,
        for<'a> S::ViewOutput: p::View<(&'a p::MenuInstance<S::Choice>, &'a S::Extra)>,
    {
        type Return = p::MenuOutput<S::Choice>;
        type Data = S::DataInput;
        type View = S::ViewInput;

        fn tick<I>(
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            _duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            let menu_instance = self.s.menu_instance_mut(data);
            let menu_view = self.s.view(view);
            if let Some(menu_output) = menu_instance.tick_with_mouse(inputs, menu_view) {
                Tick::Return(menu_output)
            } else {
                Tick::Continue(self)
            }
        }
        fn view<F, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut F)
        where
            F: p::Frame,
            R: p::ViewTransformRgb24,
        {
            let menu_view = self.s.view_mut(view);
            let menu_instance = self.s.menu_instance(data);
            let extra = self.s.extra(data);
            use p::View;
            menu_view.view((menu_instance, extra), context, frame)
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
            self,
            data: &mut Self::Data,
            inputs: I,
            view: &Self::View,
            duration: Duration,
        ) -> Tick<Self::Return, Self>
        where
            I: Iterator<Item = p::Input>,
        {
            let Self { t, mut f } = self;
            match t.tick(data, inputs, view, duration) {
                Tick::Continue(t) => Tick::Continue(Self { t, f }),
                Tick::Return(r) => f(r).map_continue(|t| Self { t, f }),
            }
        }

        fn view<G, R>(&self, data: &Self::Data, view: &mut Self::View, context: p::ViewContext<R>, frame: &mut G)
        where
            G: p::Frame,
            R: p::ViewTransformRgb24,
        {
            self.t.view(data, view, context, frame)
        }
    }

    fn inner() -> impl TickRoutine<Return = Option<Return>, Data = AppData, View = AppView> {
        let main_menu = MenuInstanceExtraRoutine::new(SelectMainMenuExtra);
        let colour_menu = MenuInstanceRoutine::new().select(SelectColourMenu);
        main_menu.and_then(|menu_output| match menu_output {
            p::MenuOutput::Quit => Either::A(Value::new(Some(Return::Quit))),
            p::MenuOutput::Cancel => Either::A(Value::new(None)),
            p::MenuOutput::Finalise(choice) => {
                match choice {
                    MainMenuChoice::ChooseColour => Either::B(colour_menu.map_side_effect(
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
                    MainMenuChoice::Quit => Either::A(Value::new(Some(Return::Quit))),
                }
            }
        })
    }

    pub fn test() -> impl TickRoutine<Return = Return, Data = AppData, View = AppView> {
        inner().repeat(|event| match event {
            Some(Return::Quit) => Tick::Return(Return::Quit),
            None => Tick::Continue(inner()),
        })
    }

    struct SelectColourMenu;
    impl ViewSelector for SelectColourMenu {
        type ViewInput = AppView;
        type ViewOutput = p::MenuInstanceView<p::MenuEntryStylePair>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.colour_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.colour_menu
        }
    }
    impl DataSelector for SelectColourMenu {
        type DataInput = AppData;
        type DataOutput = p::MenuInstance<ColourMenuChoice>;
        fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
            &input.colour_menu
        }
        fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
            &mut input.colour_menu
        }
    }
    impl Selector for SelectColourMenu {}

    struct SelectMainMenuExtra;
    impl ViewSelector for SelectMainMenuExtra {
        type ViewInput = AppView;
        type ViewOutput = p::MenuInstanceView<ChooseMenuEntryStyle<MainMenuChoice>>;

        fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
            &input.main_menu
        }
        fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
            &mut input.main_menu
        }
    }
    impl MenuInstanceExtraSelect for SelectMainMenuExtra {
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

use app::TickRoutine;
use p::Frame;
use prototty as p;
use prototty_glutin as pg;
use std::time::Instant;

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
            p::Tick::Continue(tick_routine) => tick_routine,
            p::Tick::Return(app::Return::Quit) => break,
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
