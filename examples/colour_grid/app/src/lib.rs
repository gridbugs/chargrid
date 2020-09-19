use chargrid::*;
use common_event::*;
use event_routine::*;
use render::{Rgb24, View, ViewCell};

struct AppData;
struct AppView;
struct PressAnyKey;

struct ColourSquare;
impl View<()> for ColourSquare {
    fn view<F: Frame, C: ColModify>(&mut self, (): (), context: ViewContext<C>, frame: &mut F) {
        let size = context.size;
        for y in 0..size.height() {
            for x in 0..size.width() {
                let coord = Coord::new(x as i32, y as i32);
                let r = 255 - ((x * 255) / size.width());
                let g = (x * 510) / size.width();
                let g = if g > 255 { 510 - g } else { g };
                let b = (x * 255) / size.width();
                let mul = 255 - ((y * 255) / size.height());
                let col = Rgb24::new(r as u8, g as u8, b as u8).normalised_scalar_mul(mul as u8);
                let cell = ViewCell::new()
                    .with_character(' ')
                    .with_background(col)
                    .with_foreground(col);
                frame.set_cell_relative(coord, 0, cell, context);
            }
        }
    }
}

impl EventRoutine for PressAnyKey {
    type Return = ();
    type Data = AppData;
    type View = AppView;
    type Event = CommonEvent;

    fn handle<EP>(
        self,
        _data: &mut Self::Data,
        _view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => {
                if input.is_keyboard() {
                    Handled::Return(())
                } else {
                    Handled::Continue(s)
                }
            }
            CommonEvent::Frame(_) => Handled::Continue(s),
        })
    }

    fn view<F, C>(
        &self,
        _data: &Self::Data,
        _view: &mut Self::View,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        F: Frame,
        C: ColModify,
    {
        ColourSquare.view((), context, frame);
    }
}

fn event_routine(
) -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    PressAnyKey
}

pub fn app() -> impl app::App {
    event_routine().app_one_shot_ignore_return(AppData, AppView)
}
