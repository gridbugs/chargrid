use crate::{
    MenuEntryRichString, MenuEntryToRender, MenuIndexFromScreenCoord, MenuInstance, MenuInstanceChoose,
    MenuInstanceMouseTracker,
};
use chargrid_event_routine::{event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled};
use chargrid_input::Input;
use chargrid_render::*;
use chargrid_text::StringViewSingleLine;
use std::marker::PhantomData;

pub struct DynamicStyleMenuInstanceView {
    mouse_tracker: MenuInstanceMouseTracker,
    buf: String,
}

impl DynamicStyleMenuInstanceView {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            mouse_tracker: Default::default(),
        }
    }
}

impl Default for DynamicStyleMenuInstanceView {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DynamicStyleMenuInstanceModel<'a, E, S>
where
    E: Clone,
    S: MenuEntryRichString<Entry = E>,
{
    menu_instance: &'a MenuInstance<E>,
    menu_entry_rich_string: &'a S,
}

impl<'a, E, S> View<DynamicStyleMenuInstanceModel<'a, E, S>> for DynamicStyleMenuInstanceView
where
    E: Clone,
    S: MenuEntryRichString<Entry = E>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        DynamicStyleMenuInstanceModel {
            menu_instance,
            menu_entry_rich_string,
        }: DynamicStyleMenuInstanceModel<'a, E, S>,
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
            let style = menu_entry_rich_string.render_rich_string(entry_to_render, &mut self.buf);
            let mut view = StringViewSingleLine::new(style);
            let size = view.view_size(&self.buf, context.add_offset(Coord::new(0, i as i32)), frame);
            self.mouse_tracker.on_entry_view_size(size);
        }
    }
}

impl MenuIndexFromScreenCoord for DynamicStyleMenuInstanceView {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        self.mouse_tracker.menu_index_from_screen_coord(len, coord)
    }
}

pub struct DynamicStyleMenuInstanceRoutine<C, S> {
    choose: PhantomData<C>,
    menu_entry_rich_string: S,
}
impl<C, S> DynamicStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryRichString<Entry = C::Entry>,
{
    pub fn new(menu_entry_rich_string: S) -> Self {
        Self {
            choose: PhantomData,
            menu_entry_rich_string,
        }
    }
}
impl<C, S> Clone for DynamicStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryRichString<Entry = C::Entry> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            choose: PhantomData,
            menu_entry_rich_string: self.menu_entry_rich_string.clone(),
        }
    }
}

impl<C, S> EventRoutine for DynamicStyleMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryRichString<Entry = C::Entry>,
{
    type Return = C::Output;
    type Data = C;
    type View = DynamicStyleMenuInstanceView;
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
        let model = DynamicStyleMenuInstanceModel {
            menu_instance: data.menu_instance(),
            menu_entry_rich_string: &self.menu_entry_rich_string,
        };
        view.view(model, context, frame);
    }
}
