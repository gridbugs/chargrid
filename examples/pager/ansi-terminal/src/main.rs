use pager_prototty::App;
use prototty_ansi_terminal::{col_encode, Context};
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    let context = Context::new().unwrap();
    context.run_app(App::new(text), col_encode::FromTermInfoRgb);
    Ok(())
}
