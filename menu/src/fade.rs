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

pub struct FadeMenuEntryView<E> {
    last_change: BTreeMap<E, MenuEntryChange>,
    selected_fade_in: Duration,
    selected_fade_out: Duration,
}

impl<E> FadeMenuEntryView<E>
where
    E: Ord,
{
    pub fn new() -> Self {
        Self {
            last_change: BTreeMap::new(),
            selected_fade_in: Duration::from_millis(127),
            selected_fade_out: Duration::from_millis(255),
        }
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
        let current = self
            .last_change
            .entry(entry.clone())
            .or_insert_with(|| MenuEntryChange {
                change_to: MenuEntryStatus::Normal,
                foreground: FadeInstance::constant(Rgb24::new(127, 127, 127)),
                background: FadeInstance::constant(Rgb24::new(0, 0, 0)),
            });
        match current.change_to {
            MenuEntryStatus::Normal => (),
            MenuEntryStatus::Selected => {
                current.change_to = MenuEntryStatus::Normal;
                current.foreground = FadeInstance::new(
                    current.foreground.current(*since_epoch),
                    Rgb24::new(127, 127, 127),
                    self.selected_fade_out,
                    *since_epoch,
                );
            }
        }
        let foreground = current.foreground.current(*since_epoch);
        let background = current.background.current(*since_epoch);
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(Style::new().with_foreground(foreground).with_background(background)),
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
        let current = self
            .last_change
            .entry(entry.clone())
            .or_insert_with(|| MenuEntryChange {
                change_to: MenuEntryStatus::Selected,
                foreground: FadeInstance::constant(Rgb24::new(255, 255, 255)),
                background: FadeInstance::constant(Rgb24::new(0, 0, 0)),
            });
        match current.change_to {
            MenuEntryStatus::Selected => (),
            MenuEntryStatus::Normal => {
                current.change_to = MenuEntryStatus::Selected;
                current.foreground = FadeInstance::new(
                    current.foreground.current(*since_epoch),
                    Rgb24::new(255, 255, 255),
                    self.selected_fade_in,
                    *since_epoch,
                );
                current.background = FadeInstance::new(
                    Rgb24::new(127, 127, 127),
                    Rgb24::new(0, 0, 0),
                    Duration::from_millis(255),
                    *since_epoch,
                );
            }
        }
        let foreground = current.foreground.current(*since_epoch);
        let background = current.background.current(*since_epoch);
        let s: &str = entry.into();
        menu_entry_view(
            s,
            StringViewSingleLine::new(
                Style::new()
                    .with_bold(true)
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
        view.view((data.menu_instance(), &self.since_epoch), context, frame);
    }
}
