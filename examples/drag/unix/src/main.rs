extern crate drag_prototty;
extern crate prototty_unix;

use drag_prototty::{App, AppView, Quit};
use prototty_unix::{col_encode, Context};

fn main() {
    let mut context = Context::new().unwrap();
    let mut app = App::default();
    let mut app_view = AppView;
    loop {
        context
            .render(&mut app_view, &app, col_encode::FromTermInfoRgb)
            .unwrap();
        if let Some(Quit) = app.update(context.drain_input().unwrap()) {
            break;
        }
    }
}
