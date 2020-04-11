use general_audio_native::NativeAudioPlayer;
use prototty_ansi_terminal::{col_encode, Context};
use soundboard_prototty::app;

fn main() {
    let player = NativeAudioPlayer::new_default_device();
    let context = Context::new().unwrap();
    context.run_app(app(player), col_encode::FromTermInfoRgb);
}
