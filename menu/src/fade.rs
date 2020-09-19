use crate::{
    MenuEntryString, MenuEntryToRender, MenuIndexFromScreenCoord, MenuInstance, MenuInstanceChoose,
    MenuInstanceMouseTracker, Selected,
};
use chargrid_event_routine::{
    common_event, event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled,
};
use chargrid_render::{ColModify, Coord, Frame, Rgb24, Style, View, ViewContext};
use chargrid_text::StringViewSingleLine;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

fn duration_ratio_u8(delta: Duration, period: Duration) -> Result<u8, Duration> {
    if let Some(remaining) = delta.checked_sub(period) {
        Err(remaining)
    } else {
        Ok(((delta.as_micros() * 255) / period.as_micros()) as u8)
    }
}

struct FadeInstance {
    from: Rgb24,
    to: Rgb24,
    started_at_since_epoch: Duration,
    total_duration: Duration,
}

impl FadeInstance {
    fn new(from: Rgb24, to: Rgb24, total_duration: Duration, since_epoch: Duration) -> Self {
        Self {
            from,
            to,
            started_at_since_epoch: since_epoch,
            total_duration,
        }
    }

    fn constant(col: Rgb24) -> Self {
        Self::new(col, col, Duration::from_millis(0), Duration::from_millis(0))
    }

    fn current(&self, since_epoch: Duration) -> Rgb24 {
        if let Some(time_delta) = since_epoch.checked_sub(self.started_at_since_epoch) {
            match duration_ratio_u8(time_delta, self.total_duration) {
                Ok(ratio) => self.from.linear_interpolate(self.to, ratio),
                Err(_) => self.to,
            }
        } else {
            self.from
        }
    }

    fn transform_foreground(&self, style: &fade_spec::Style, since_epoch: Duration) -> Self {
        let from = match style.from.foreground {
            fade_spec::FromCol::Current => self.current(since_epoch),
            fade_spec::FromCol::Rgb24(rgb24) => rgb24,
        };
        Self::new(
            from,
            style.to.foreground,
            style.durations.foreground,
            since_epoch,
        )
    }

    fn transform_background(&self, style: &fade_spec::Style, since_epoch: Duration) -> Self {
        let from = match style.from.background {
            fade_spec::FromCol::Current => self.current(since_epoch),
            fade_spec::FromCol::Rgb24(rgb24) => rgb24,
        };
        Self::new(
            from,
            style.to.background,
            style.durations.background,
            since_epoch,
        )
    }
}

struct MenuEntryChange {
    change_to: Option<Selected>,
    foreground: FadeInstance,
    background: FadeInstance,
}

pub mod fade_spec {
    pub use chargrid_render::Rgb24;
    pub use std::time::Duration;

    #[derive(Debug, Clone)]
    pub enum FromCol {
        Current,
        Rgb24(Rgb24),
    }

    #[derive(Debug, Clone)]
    pub struct From {
        pub foreground: FromCol,
        pub background: FromCol,
    }
    impl From {
        pub fn current() -> Self {
            Self {
                foreground: FromCol::Current,
                background: FromCol::Current,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct To {
        pub foreground: Rgb24,
        pub background: Rgb24,
        pub bold: bool,
        pub underline: bool,
    }

    #[derive(Debug, Clone)]
    pub struct Durations {
        pub foreground: Duration,
        pub background: Duration,
    }

    #[derive(Debug, Clone)]
    pub struct Style {
        pub to: To,
        pub from: From,
        pub durations: Durations,
    }

    #[derive(Debug, Clone)]
    pub struct Spec {
        pub selected: Style,
        pub normal: Style,
    }
}

pub struct FadeMenuInstanceView {
    last_change: HashMap<usize, MenuEntryChange>,
    previous_view_since_epoch: Duration,
    spec: fade_spec::Spec,
    mouse_tracker: MenuInstanceMouseTracker,
    buf: String,
}

impl FadeMenuInstanceView {
    pub fn new(spec: fade_spec::Spec) -> Self {
        Self {
            last_change: HashMap::new(),
            previous_view_since_epoch: Duration::from_millis(0),
            spec,
            mouse_tracker: Default::default(),
            buf: String::new(),
        }
    }
    pub fn clear(&mut self) {
        self.last_change.clear();
    }
}

pub struct FadeMenuInstanceModel<'a, E, S>
where
    E: Clone,
    S: MenuEntryString<Entry = E>,
{
    menu_instance: &'a MenuInstance<E>,
    menu_entry_string: &'a S,
    since_epoch: Duration,
}

impl<'a, E, S> View<FadeMenuInstanceModel<'a, E, S>> for FadeMenuInstanceView
where
    E: Clone,
    S: MenuEntryString<Entry = E>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        FadeMenuInstanceModel {
            menu_instance,
            menu_entry_string,
            since_epoch,
        }: FadeMenuInstanceModel<'a, E, S>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.mouse_tracker.new_frame(context.offset);
        let spec = &self.spec;
        for (i, entry, maybe_selected) in menu_instance.enumerate() {
            let current_style = if let Some(Selected) = maybe_selected {
                &spec.selected
            } else {
                &spec.normal
            };
            let current = self
                .last_change
                .entry(i)
                .or_insert_with(|| MenuEntryChange {
                    change_to: maybe_selected,
                    foreground: FadeInstance::constant(current_style.to.foreground),
                    background: FadeInstance::constant(current_style.to.background),
                });
            match (current.change_to, maybe_selected) {
                (None, None) | (Some(Selected), Some(Selected)) => (),
                (Some(Selected), None) => {
                    current.change_to = None;
                    current.foreground = current
                        .foreground
                        .transform_foreground(&spec.normal, since_epoch);
                    current.background = current
                        .background
                        .transform_background(&spec.normal, since_epoch);
                }
                (None, Some(Selected)) => {
                    current.change_to = Some(Selected);
                    current.foreground = current
                        .foreground
                        .transform_foreground(&spec.selected, since_epoch);
                    current.background = current
                        .background
                        .transform_background(&spec.selected, since_epoch);
                }
            }
            let foreground = current.foreground.current(since_epoch);
            let background = current.background.current(since_epoch);
            let entry_to_render = MenuEntryToRender {
                index: i,
                entry,
                selected: maybe_selected.is_some(),
            };
            self.buf.clear();
            menu_entry_string.render_string(entry_to_render, &mut self.buf);
            let mut view = StringViewSingleLine::new(
                Style::new()
                    .with_bold(spec.normal.to.bold)
                    .with_underline(spec.normal.to.underline)
                    .with_foreground(foreground)
                    .with_background(background),
            );
            let size = view.view_size(
                &self.buf,
                context.add_offset(Coord::new(0, i as i32)),
                frame,
            );
            self.mouse_tracker.on_entry_view_size(size);
        }
    }
}

impl MenuIndexFromScreenCoord for FadeMenuInstanceView {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        self.mouse_tracker.menu_index_from_screen_coord(len, coord)
    }
}

pub struct FadeMenuInstanceRoutine<C, S> {
    choose: PhantomData<C>,
    since_epoch: Duration,
    menu_entry_string: S,
}

impl<C, S> FadeMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    S: MenuEntryString<Entry = C::Entry>,
{
    pub fn new(menu_entry_string: S) -> Self {
        Self {
            choose: PhantomData,
            since_epoch: Duration::from_millis(0),
            menu_entry_string,
        }
    }
}

impl<C, T> Clone for FadeMenuInstanceRoutine<C, T>
where
    C: MenuInstanceChoose,
    T: MenuEntryString<Entry = C::Entry> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            choose: PhantomData,
            since_epoch: self.since_epoch,
            menu_entry_string: self.menu_entry_string.clone(),
        }
    }
}

impl<C, S> EventRoutine for FadeMenuInstanceRoutine<C, S>
where
    C: MenuInstanceChoose,
    C::Entry: Clone,
    S: MenuEntryString<Entry = C::Entry>,
{
    type Return = C::Output;
    type Data = C;
    type View = FadeMenuInstanceView;
    type Event = common_event::CommonEvent;

    fn handle<EP>(
        self,
        data: &mut Self::Data,
        view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            common_event::CommonEvent::Input(input) => {
                if let Some(menu_output) = data.choose(view, input) {
                    Handled::Return(menu_output)
                } else {
                    Handled::Continue(s)
                }
            }
            common_event::CommonEvent::Frame(duration) => {
                let Self {
                    choose,
                    since_epoch,
                    menu_entry_string,
                } = s;
                Handled::Continue(Self {
                    choose,
                    since_epoch: since_epoch + duration,
                    menu_entry_string,
                })
            }
        })
    }

    fn view<F, CM>(
        &self,
        data: &Self::Data,
        view: &mut Self::View,
        context: ViewContext<CM>,
        frame: &mut F,
    ) where
        F: Frame,
        CM: ColModify,
    {
        if self.since_epoch < view.previous_view_since_epoch {
            view.clear();
        }
        let model = FadeMenuInstanceModel {
            menu_instance: data.menu_instance(),
            menu_entry_string: &self.menu_entry_string,
            since_epoch: self.since_epoch,
        };
        view.view(model, context, frame);
        view.previous_view_since_epoch = self.since_epoch;
    }
}
