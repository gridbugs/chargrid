use prototty_graphical::*;
use prototty_native_audio::NativeAudioPlayer;
use soundboard_prototty::app;

fn main() {
    let player = NativeAudioPlayer::new_default_device();
    let context = Context::new(ContextDescriptor {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Soundboard".to_string(),
        window_dimensions: Dimensions {
            width: 640.,
            height: 480.,
        },
        cell_dimensions: Dimensions {
            width: 16.,
            height: 16.,
        },
        font_dimensions: Dimensions {
            width: 16.,
            height: 16.,
        },
        font_source_dimensions: Dimensions {
            width: 16.,
            height: 16.,
        },
        underline_width: 0.1,
        underline_top_offset: 0.8,
    })
    .unwrap();
    context.run_app(app(player));
}
