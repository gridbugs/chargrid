use crate::{MenuIndexFromScreenCoord, MenuInstanceChoose};
use chargrid_event_routine::{
    event_or_peek_with_handled, DataSelector, EventOrPeek, EventRoutine, Handled, ViewSelector,
};
use chargrid_input::Input;
use chargrid_render::{ColModify, Frame, View, ViewContext};

pub trait ChooseSelector: DataSelector {
    type ChooseOutput: MenuInstanceChoose;
    fn choose_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::ChooseOutput;
}

pub struct MenuInstanceRoutine<S> {
    s: S,
}
impl<S> MenuInstanceRoutine<S>
where
    S: DataSelector + ViewSelector + ChooseSelector,
    S::ViewOutput: MenuIndexFromScreenCoord,
    for<'a> S::ViewOutput: View<&'a S::DataOutput>,
{
    pub fn new(s: S) -> Self {
        Self { s }
    }
}

impl<S> EventRoutine for MenuInstanceRoutine<S>
where
    S: ViewSelector + ChooseSelector,
    S::ViewOutput: MenuIndexFromScreenCoord,
    for<'a> S::ViewOutput: View<&'a S::DataOutput>,
{
    type Return = <S::ChooseOutput as MenuInstanceChoose>::Output;
    type Data = S::DataInput;
    type View = S::ViewInput;
    type Event = Input;

    fn handle<EP>(
        self,
        data: &mut Self::Data,
        view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
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

    fn view<F, C>(
        &self,
        data: &Self::Data,
        view: &mut Self::View,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        F: Frame,
        C: ColModify,
    {
        let view = self.s.view_mut(view);
        let data = self.s.data(data);
        view.view(data, context, frame)
    }
}
