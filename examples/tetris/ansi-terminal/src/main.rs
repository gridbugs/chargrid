use prototty_ansi_terminal::{col_encode, Context};
use tetris_prototty::TetrisApp;

fn main() {
    let context = Context::new().unwrap();
    let app = TetrisApp::new(rand::thread_rng());
    context.run_app(app, col_encode::FromTermInfoRgb);
}
