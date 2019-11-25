use prototty_graphical_::*;
use tetris_prototty::TetrisApp;

fn main() {
    env_logger::init();
    let context = Context::new(ContextDescription {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Tetris".to_string(),
        window_dimensions: WindowDimensions::Windowed(Dimensions {
            width: 640.,
            height: 480.,
        }),
        cell_dimensions: Dimensions {
            width: 32.,
            height: 32.,
        },
        font_dimensions: Dimensions {
            width: 32.,
            height: 32.,
        },
        underline_width: 4.,
        underline_bottom_offset: 2.,
    })
    .unwrap();
    let app = TetrisApp::new(rand::thread_rng());
    context.run_app(app);
}
