use chargrid_ansi_terminal::{col_encode, Context};
use tetris_app::app;

fn main() {
    let context = Context::new().unwrap();
    context.run(app(rand::thread_rng()), col_encode::FromTermInfoRgb);
}
