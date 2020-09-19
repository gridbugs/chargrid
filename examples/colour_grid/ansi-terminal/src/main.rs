use chargrid_ansi_terminal::{col_encode, Context};
use colour_grid_app::app;
use simon::*;

#[derive(Clone)]
enum ColEncodeChoice {
    TrueColour,
    Rgb,
    Greyscale,
    Ansi,
}

impl ColEncodeChoice {
    fn arg() -> impl Arg<Item = Self> {
        use ColEncodeChoice::*;
        (args_choice! {
            flag("", "true-colour", "").some_if(TrueColour),
            flag("", "rgb", "").some_if(Rgb),
            flag("", "greyscale", "").some_if(Greyscale),
            flag("", "ansi", "").some_if(Ansi),
        })
        .with_default(Rgb)
    }
}

fn main() {
    let col_encode_choice = ColEncodeChoice::arg()
        .with_help_default()
        .parse_env_or_exit();
    let context = Context::new().unwrap();
    let app = app();
    use ColEncodeChoice as C;
    match col_encode_choice {
        C::TrueColour => context.run_app(app, col_encode::XtermTrueColour),
        C::Rgb => context.run_app(app, col_encode::FromTermInfoRgb),
        C::Greyscale => context.run_app(app, col_encode::FromTermInfoGreyscale),
        C::Ansi => context.run_app(app, col_encode::FromTermInfoAnsi16Colour),
    }
}
