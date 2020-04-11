use general_audio_native::NativeAudioPlayer;
use chargrid_ansi_terminal::{col_encode, Context};
use soundboard_app::app;

fn main() {
    let player = NativeAudioPlayer::new_default_device();
    let context = Context::new().unwrap();
    context.run_app(app(player), col_encode::FromTermInfoRgb);
}
