use chargrid_graphical::*;
use general_audio_native::NativeAudioPlayer;
use soundboard_app::app;

fn main() {
    env_logger::init();
    let player = NativeAudioPlayer::new_default_device();
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Soundboard".to_string(),
        window_dimensions_px: Dimensions {
            width: 640.,
            height: 480.,
        },
        cell_dimensions_px: Dimensions {
            width: 16.,
            height: 16.,
        },
        font_scale: Dimensions {
            width: 16.,
            height: 16.,
        },
        underline_width_cell_ratio: 0.1,
        underline_top_offset_cell_ratio: 0.8,
        resizable: false,
    });
    context.run_app(app(player));
}
