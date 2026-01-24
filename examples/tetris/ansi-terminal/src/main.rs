use chargrid_ansi_terminal::{Context, col_encode};
use tetris_app::app;

fn main() {
    let context = Context::new().unwrap();
    context.run(app(rand::rng()), col_encode::FromTermInfoRgb);
}
