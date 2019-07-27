use prototty_unix::{encode_colour, Context};
use roguelike_prototty::{event_routine, AppData, AppView};
use std::time::Duration;

fn main() {
    Context::new()
        .unwrap()
        .into_runner(Duration::from_millis(16))
        .run(
            event_routine(),
            &mut AppData::new(),
            &mut AppView::new(),
            encode_colour::FromTermInfoRgb,
        )
        .unwrap()
}
