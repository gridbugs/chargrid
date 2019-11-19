use prototty_ansi_terminal::{col_encode, Context};
use prototty_native_audio::NativeAudioPlayer;
use soundboard_prototty::{event_routine, AppData, AppView};
use std::time::Duration;

fn main() {
    let player = NativeAudioPlayer::new_default_device();
    let mut runner = Context::new().unwrap().into_runner(Duration::from_millis(16));
    let mut app_data = AppData::new(player);
    let mut app_view = AppView::new();
    runner
        .run(
            event_routine(),
            &mut app_data,
            &mut app_view,
            col_encode::FromTermInfoRgb,
        )
        .unwrap();
}
