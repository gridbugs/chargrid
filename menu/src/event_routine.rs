use crate::{MenuIndexFromScreenCoord, MenuInstance, MenuInstanceChoose};
use prototty_event_routine::{event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled, ViewSelector};
use prototty_input::Input;
use prototty_render::{ColModify, Frame, View, ViewContext};
use std::marker::PhantomData;

#[derive(Default)]
pub struct MenuInstanceRoutine<V, C> {
    view: PhantomData<V>,
    choose: PhantomData<C>,
}
impl<V, C> MenuInstanceRoutine<V, C>
where
    C: MenuInstanceChoose,
    for<'a> V: View<&'a MenuInstance<C::Entry>>,
    V: MenuIndexFromScreenCoord,
{
    pub fn new() -> Self {
        Self {
            view: PhantomData,
            choose: PhantomData,
        }
    }
}
impl<V, C> Clone for MenuInstanceRoutine<V, C>
where
    C: MenuInstanceChoose,
    for<'a> V: View<&'a MenuInstance<C::Entry>>,
    V: MenuIndexFromScreenCoord,
{
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<V, C> Copy for MenuInstanceRoutine<V, C>
where
    C: MenuInstanceChoose,
    for<'a> V: View<&'a MenuInstance<C::Entry>>,
    V: MenuIndexFromScreenCoord,
{
}

impl<V, C> EventRoutine for MenuInstanceRoutine<V, C>
where
    C: MenuInstanceChoose,
    for<'a> V: View<&'a MenuInstance<C::Entry>>,
    V: MenuIndexFromScreenCoord,
{
    type Return = C::Output;
    type Data = C;
    type View = V;
    type Event = Input;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| {
            if let Some(menu_output) = data.choose(view, event) {
                Handled::Return(menu_output)
            } else {
                Handled::Continue(s)
            }
        })
    }

    fn view<F, CM>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<CM>, frame: &mut F)
    where
        F: Frame,
        CM: ColModify,
    {
        view.view(data.menu_instance(), context, frame);
    }
}

pub trait MenuInstanceExtraSelect {
    type DataInput;
    type Extra;
    type Choose: MenuInstanceChoose;
    fn choose<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Choose;
    fn choose_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::Choose;
    fn extra<'a>(&self, input: &'a Self::DataInput) -> &'a Self::Extra;
}

pub struct MenuInstanceExtraRoutine<S> {
    s: S,
}
impl<S> MenuInstanceExtraRoutine<S>
where
    S: MenuInstanceExtraSelect + ViewSelector,
    S::ViewOutput: MenuIndexFromScreenCoord,
    for<'a> S::ViewOutput: View<(&'a MenuInstance<<S::Choose as MenuInstanceChoose>::Entry>, &'a S::Extra)>,
{
    pub fn new(s: S) -> Self {
        Self { s }
    }
}

impl<S> EventRoutine for MenuInstanceExtraRoutine<S>
where
    S: MenuInstanceExtraSelect + ViewSelector,
    S::ViewOutput: MenuIndexFromScreenCoord,
    for<'a> S::ViewOutput: View<(&'a MenuInstance<<S::Choose as MenuInstanceChoose>::Entry>, &'a S::Extra)>,
{
    type Return = <S::Choose as MenuInstanceChoose>::Output;
    type Data = S::DataInput;
    type View = S::ViewInput;
    type Event = Input;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| {
            let choose = s.s.choose_mut(data);
            let menu_view = s.s.view(view);
            if let Some(menu_output) = choose.choose(menu_view, event) {
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
        let menu_instance = self.s.choose(data).menu_instance();
        let extra = self.s.extra(data);
        menu_view.view((menu_instance, extra), context, frame)
    }
}
