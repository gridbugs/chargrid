use drag_app::App;
use chargrid_ansi_terminal::{col_encode, Context};

fn main() {
    let context = Context::new().unwrap();
    context.run_app(App::default(), col_encode::FromTermInfoRgb);
}
