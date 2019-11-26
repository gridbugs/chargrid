use prototty_ansi_terminal::{col_encode, Context};
use prototty_native_audio::NativeAudioPlayer;
use soundboard_prototty::app;

fn main() {
    let player = NativeAudioPlayer::new_default_device();
    let context = Context::new().unwrap();
    context.run_app(app(player), col_encode::FromTermInfoRgb);
}
