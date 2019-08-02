use prototty_unix::{col_encode, Context};
use roguelike_prototty::{event_routine, AppData, AppView, Controls};
use std::time::Duration;

fn main() {
    Context::new()
        .unwrap()
        .into_runner(Duration::from_millis(16))
        .run(
            event_routine(),
            &mut AppData::new(Controls::default()),
            &mut AppView::new(),
            col_encode::FromTermInfoRgb,
        )
        .unwrap()
}
