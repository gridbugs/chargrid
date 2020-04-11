use colour_picker_app::app;
use chargrid_ansi_terminal::{col_encode, Context};

fn main() {
    let context = Context::new().unwrap();
    context.run_app(app(), col_encode::FromTermInfoRgb);
}
