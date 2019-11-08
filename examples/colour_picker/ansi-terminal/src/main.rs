use colour_picker_prototty as app;
use prototty_ansi_terminal as pu;
use std::time::Duration;

fn main() {
    let mut runner = pu::Context::new().unwrap().into_runner(Duration::from_millis(16));
    runner
        .run(
            app::test(),
            &mut app::AppData::new(),
            &mut app::AppView::new(),
            pu::col_encode::FromTermInfoRgb,
        )
        .unwrap();
}
