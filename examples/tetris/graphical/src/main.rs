use prototty_graphical_::*;
use tetris_prototty::TetrisApp;

fn main() {
    env_logger::init();
    let context = ContextBuilder::new_with_font_bytes(FontBytes {
        normal: include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
        bold: include_bytes!("fonts/PxPlus_IBM_CGA.ttf").to_vec(),
    })
    .with_window_dimensions(WindowDimensions::Windowed(Dimensions {
        width: 640.,
        height: 400.,
    }))
    .with_font_dimensions(Dimensions {
        width: 16.0,
        height: 16.0,
    })
    .with_cell_dimensions(Dimensions {
        width: 16.0,
        height: 16.0,
    })
    .with_underline_bottom_offset(14.)
    .with_underline_width(4.)
    .build()
    .unwrap();
    let app = TetrisApp::new(rand::thread_rng());
    context.run_app(app);
}
