use chargrid_ansi_terminal::{col_encode, Context};
use component_experiment_app::app;

fn main() {
    let context = Context::new().unwrap();
    let grid_size = context.size().unwrap();
    context.run_app(app(grid_size), col_encode::FromTermInfoRgb);
}
