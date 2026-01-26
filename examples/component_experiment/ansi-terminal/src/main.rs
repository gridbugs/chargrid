use chargrid_ansi_terminal::{Context, col_encode};
use component_experiment_app::app;

fn main() {
    let context = Context::new().unwrap();
    context.run(app(), col_encode::FromTermInfoRgb);
}
