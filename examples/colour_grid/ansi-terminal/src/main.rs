use chargrid_ansi_terminal::{col_encode, Context};
use colour_grid_app::app;

enum ColEncodeChoice {
    TrueColour,
    Rgb,
    Greyscale,
    Ansi,
}

impl ColEncodeChoice {
    fn parser() -> impl meap::Parser<Item = Self> {
        use ColEncodeChoice::*;
        meap::choose_at_most_one!(
            flag("true-colour").some_if(TrueColour),
            flag("rgb").some_if(Rgb),
            flag("greyscale").some_if(Greyscale),
            flag("ansi").some_if(Ansi),
        )
        .with_default_general(TrueColour)
    }
}

fn main() {
    use meap::Parser;
    let col_encode_choice = ColEncodeChoice::parser()
        .with_help_default()
        .parse_env_or_exit();
    let context = Context::new().unwrap();
    let app = app();
    use ColEncodeChoice as C;
    match col_encode_choice {
        C::TrueColour => context.run(app, col_encode::XtermTrueColour),
        C::Rgb => context.run(app, col_encode::FromTermInfoRgb),
        C::Greyscale => context.run(app, col_encode::FromTermInfoGreyscale),
        C::Ansi => context.run(app, col_encode::FromTermInfoAnsi16Colour),
    }
}
