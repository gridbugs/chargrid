use crate::{
    MenuEntryString, MenuEntryToRender, MenuIndexFromScreenCoord, MenuInstance, MenuInstanceChoose,
    MenuInstanceMouseTracker, Selected,
};
use chargrid_event_routine::{event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled};
use chargrid_input::Input;
use chargrid_render::*;
use chargrid_text::StringViewSingleLine;
use std::marker::PhantomData;

pub struct StaticStyleMenuInstanceView {
    mouse_tracker: MenuInstanceMouseTracker,
    buf: String,
    selected: Style,
    normal: Style,
}

impl StaticStyleMenuInstanceView {
    pub fn new(normal: Style, selected: Style) -> Self {
        Self {
            normal,
            selected,
            buf: String::new(),
            mouse_tracker: Default::default(),
        }
    }
}

pub struct StaticStyleMenuInstanceModel<'a, E, S>
where
    E: Clone,
    S: MenuEntryString<Entry = E>,
{
    menu_instance: &'a MenuInstance<E>,
    menu_entry_string: &'a S,
}

impl<'a, E, S> View<StaticStyleMenuInstanceModel<'a, E, S>> for StaticStyleMenuInstanceView
where
    E: Clone,
    S: MenuEntryString<Entry = E>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        StaticStyleMenuInstanceModel {
            menu_instance,
            menu_entry_string,
        }: StaticStyleMenuInstanceModel<'a, E, S>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.mouse_tracker.new_frame(context.offset);
        for (i, entry, maybe_selected) in menu_instance.enumerate() {
            self.buf.clear();
            let entry_to_render = MenuEntryToRender {
                index: i,
                entry,
                selected: maybe_selected.is_some(),
            };
            menu_entry_string.render_string(entry_to_render, &mut self.buf);
            let style = if let Some(Selected) = maybe_selected {
                self.selected
            } else {
                self.normal
            };
            let mut view = StringViewSingleLine::new(style);
            let size = view.view_size(&self.buf, context.add_offset(Coord::new(0, i as i32)), frame);
            self.mouse_tracker.on_entry_view_size(size);
        }
    }
}

impl MenuIndexFromScreenCoord for StaticStyleMenuInstanceView {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        self.mouse_tracker.menu_index_from_screen_coord(len, coord)
    }
}

pub struct StaticStyleMenuInstanceRoutine<C, S> {
    choose: PhantomData<C>,
    menu_entry_string: S,
}
impl<C, S> StaticStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryString<Entry = C::Entry>,
{
    pub fn new(menu_entry_string: S) -> Self {
        Self {
            choose: PhantomData,
            menu_entry_string,
        }
    }
}
impl<C, S> Clone for StaticStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryString<Entry = C::Entry> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            choose: PhantomData,
            menu_entry_string: self.menu_entry_string.clone(),
        }
    }
}

impl<C, S> EventRoutine for StaticStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryString<Entry = C::Entry> + Clone,
{
    type Return = C::Output;
    type Data = C;
    type View = StaticStyleMenuInstanceView;
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
        let model = StaticStyleMenuInstanceModel {
            menu_instance: data.menu_instance(),
            menu_entry_string: &self.menu_entry_string,
        };
        view.view(model, context, frame);
    }
}
