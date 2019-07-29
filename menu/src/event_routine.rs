use crate::{MenuIndexFromScreenCoord, MenuInstance, MenuOutput};
use prototty_event_routine::{event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled, ViewSelector};
use prototty_input::Input;
use prototty_render::{ColModify, Frame, View, ViewContext};
use std::marker::PhantomData;

pub struct MenuInstanceRoutine<C, V> {
    choice: PhantomData<C>,
    view: PhantomData<V>,
}
impl<C, V> MenuInstanceRoutine<C, V>
where
    C: Clone,
    for<'a> V: View<&'a MenuInstance<C>>,
{
    pub fn new() -> Self {
        Self {
            choice: PhantomData,
            view: PhantomData,
        }
    }
}
impl<C, V> Clone for MenuInstanceRoutine<C, V>
where
    C: Clone,
    for<'a> V: View<&'a MenuInstance<C>>,
{
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<C, V> Copy for MenuInstanceRoutine<C, V>
where
    C: Clone,
    for<'a> V: View<&'a MenuInstance<C>>,
{
}

impl<T, V> EventRoutine for MenuInstanceRoutine<T, V>
where
    T: Clone,
    for<'a> V: View<&'a MenuInstance<T>>,
    V: MenuIndexFromScreenCoord,
{
    type Return = MenuOutput<T>;
    type Data = MenuInstance<T>;
    type View = V;
    type Event = Input;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| {
            if let Some(menu_output) = data.handle_input(view, event) {
                Handled::Return(menu_output)
            } else {
                Handled::Continue(s)
            }
        })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        view.view(&data, context, frame);
    }
}

pub trait MenuInstanceExtraSelect {
    type DataInput;
    type Choice: Clone;
    type Extra;
    fn menu_instance<'a>(&self, input: &'a Self::DataInput) -> &'a MenuInstance<Self::Choice>;
    fn menu_instance_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut MenuInstance<Self::Choice>;
    fn extra<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Extra;
}

pub struct MenuInstanceExtraRoutine<S> {
    s: S,
}
impl<S> MenuInstanceExtraRoutine<S>
where
    S: MenuInstanceExtraSelect,
{
    pub fn new(s: S) -> Self {
        Self { s }
    }
}

impl<S> EventRoutine for MenuInstanceExtraRoutine<S>
where
    S: MenuInstanceExtraSelect + ViewSelector,
    S::ViewOutput: MenuIndexFromScreenCoord,
    for<'a> S::ViewOutput: View<(&'a MenuInstance<S::Choice>, &'a S::Extra)>,
{
    type Return = MenuOutput<S::Choice>;
    type Data = S::DataInput;
    type View = S::ViewInput;
    type Event = Input;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| {
            let menu_instance = s.s.menu_instance_mut(data);
            let menu_view = s.s.view(view);
            if let Some(menu_output) = menu_instance.handle_input(menu_view, event) {
                Handled::Return(menu_output)
            } else {
                Handled::Continue(s)
            }
        })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        let menu_view = self.s.view_mut(view);
        let menu_instance = self.s.menu_instance(data);
        let extra = self.s.extra(data);
        menu_view.view((menu_instance, extra), context, frame)
    }
}
