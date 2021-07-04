use chargrid_ggez::*;
use tetris_app::TetrisApp;

fn main() {
    env_logger::init();
    let app = TetrisApp::new(rand::thread_rng());
    let context = Context::new(ContextDescriptor {
        window_title: "Tetris".to_string(),
        window_width: 640,
        window_height: 480,
        cell_width: 20.,
        cell_height: 20.,
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        font_size: 16,
        resizable: false,
    });
    context.run_app(app);
}
