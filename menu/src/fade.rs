use crate::{menu_entry_view, MenuEntryExtraView, MenuEntryViewInfo, MenuInstanceChoose, MenuInstanceView};
use prototty_event_routine::{common_event, event_or_peek_with_handled, EventOrPeek, EventRoutine, Handled};
use prototty_render::{ColModify, Frame, Rgb24, Style, View, ViewContext};
use prototty_text::StringViewSingleLine;
use std::collections::BTreeMap;
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
        Self::new(from, style.to.foreground, style.durations.foreground, since_epoch)
    }

    fn transform_background(&self, style: &fade_spec::Style, since_epoch: Duration) -> Self {
        let from = match style.from.background {
            fade_spec::FromCol::Current => self.current(since_epoch),
            fade_spec::FromCol::Rgb24(rgb24) => rgb24,
        };
        Self::new(from, style.to.background, style.durations.background, since_epoch)
    }
}

#[derive(Clone, Copy)]
enum MenuEntryStatus {
    Selected,
    Normal,
}

struct MenuEntryChange {
    change_to: MenuEntryStatus,
    foreground: FadeInstance,
    background: FadeInstance,
}

pub mod fade_spec {
    pub use prototty_render::Rgb24;
    pub use std::time::Duration;

    pub enum FromCol {
        Current,
        Rgb24(Rgb24),
    }

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

    pub struct To {
        pub foreground: Rgb24,
        pub background: Rgb24,
        pub bold: bool,
        pub underline: bool,
    }

    pub struct Durations {
        pub foreground: Duration,
        pub background: Duration,
    }

    pub struct Style {
        pub to: To,
        pub from: From,
        pub durations: Durations,
    }

    pub struct Spec {
        pub selected: Style,
        pub normal: Style,
    }
}

pub struct FadeMenuEntryView<E> {
    last_change: BTreeMap<E, MenuEntryChange>,
    previous_view_since_epoch: Duration,
    spec: fade_spec::Spec,
}

impl<E> FadeMenuEntryView<E>
where
    E: Ord,
{
    pub fn new(spec: fade_spec::Spec) -> Self {
        Self {
            last_change: BTreeMap::new(),
            previous_view_since_epoch: Duration::from_millis(0),
            spec,
        }
    }
    pub fn clear(&mut self) {
        self.last_change.clear();
    }
}

impl<E> MenuEntryExtraView<E> for FadeMenuEntryView<E>
where
    E: Ord + Clone,
    for<'a> &'a E: Into<&'a str>,
{
    type Extra = Duration;
    fn normal<G: Frame, C: ColModify>(
        &mut self,
        entry: &E,
        since_epoch: &Duration,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        let spec = &self.spec;
        let current = self
            .last_change
            .entry(entry.clone())
            .or_insert_with(|| MenuEntryChange {
                change_to: MenuEntryStatus::Normal,
                foreground: FadeInstance::constant(spec.normal.to.foreground),
                background: FadeInstance::constant(spec.normal.to.background),
            });
        match current.change_to {
            MenuEntryStatus::Normal => (),
            MenuEntryStatus::Selected => {
                current.change_to = MenuEntryStatus::Normal;
                current.foreground = current.foreground.transform_foreground(&spec.normal, *since_epoch);
                current.background = current.background.transform_background(&spec.normal, *since_epoch);
            }
        }
        let foreground = current.foreground.current(*since_epoch);
        let background = current.background.current(*since_epoch);
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(
                Style::new()
                    .with_bold(spec.normal.to.bold)
                    .with_underline(spec.normal.to.underline)
                    .with_foreground(foreground)
                    .with_background(background),
            ),
            context,
            frame,
        )
    }
    fn selected<G: Frame, C: ColModify>(
        &mut self,
        entry: &E,
        since_epoch: &Duration,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        let spec = &self.spec;
        let current = self
            .last_change
            .entry(entry.clone())
            .or_insert_with(|| MenuEntryChange {
                change_to: MenuEntryStatus::Selected,
                foreground: FadeInstance::constant(spec.selected.to.foreground),
                background: FadeInstance::constant(spec.selected.to.background),
            });
        match current.change_to {
            MenuEntryStatus::Selected => (),
            MenuEntryStatus::Normal => {
                current.change_to = MenuEntryStatus::Selected;
                current.foreground = current.foreground.transform_foreground(&spec.selected, *since_epoch);
                current.background = current.background.transform_background(&spec.selected, *since_epoch);
            }
        }
        let foreground = current.foreground.current(*since_epoch);
        let background = current.background.current(*since_epoch);
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(
                Style::new()
                    .with_bold(spec.selected.to.bold)
                    .with_underline(spec.selected.to.underline)
                    .with_foreground(foreground)
                    .with_background(background),
            ),
            context,
            frame,
        )
    }
}

pub struct FadeMenuInstanceRoutine<C> {
    choose: PhantomData<C>,
    since_epoch: Duration,
}

impl<C> FadeMenuInstanceRoutine<C>
where
    C: MenuInstanceChoose,
{
    pub fn new() -> Self {
        Self {
            choose: PhantomData,
            since_epoch: Duration::from_millis(0),
        }
    }
}

impl<C> Clone for FadeMenuInstanceRoutine<C>
where
    C: MenuInstanceChoose,
{
    fn clone(&self) -> Self {
        Self {
            choose: PhantomData,
            since_epoch: self.since_epoch,
        }
    }
}

impl<C> Copy for FadeMenuInstanceRoutine<C> where C: MenuInstanceChoose {}

impl<C> Default for FadeMenuInstanceRoutine<C>
where
    C: MenuInstanceChoose,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> EventRoutine for FadeMenuInstanceRoutine<C>
where
    C: MenuInstanceChoose,
    C::Entry: Ord + Clone,
    for<'a> &'a C::Entry: Into<&'a str>,
{
    type Return = C::Output;
    type Data = C;
    type View = MenuInstanceView<FadeMenuEntryView<C::Entry>>;
    type Event = common_event::CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
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
                let Self { choose, since_epoch } = s;
                Handled::Continue(Self {
                    choose,
                    since_epoch: since_epoch + duration,
                })
            }
        })
    }

    fn view<F, CM>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<CM>, frame: &mut F)
    where
        F: Frame,
        CM: ColModify,
    {
        if self.since_epoch < view.entry_view.previous_view_since_epoch {
            view.entry_view.clear();
        }
        view.view((data.menu_instance(), &self.since_epoch), context, frame);
        view.entry_view.previous_view_since_epoch = self.since_epoch;
    }
}
