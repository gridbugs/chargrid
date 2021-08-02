use chargrid_ansi_terminal::{col_encode, Context};
use component_experiment_app::app;

fn main() {
    let context = Context::new().unwrap();
    context.run(app(), col_encode::FromTermInfoRgb);
}
