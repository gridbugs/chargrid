use colour_grid_prototty::{event_routine, AppData, AppView};
use prototty_unix::{col_encode, ColEncode, Context, EventRoutineRunner};
use simon::*;
use std::time::Duration;

#[derive(Clone)]
enum ColEncodeChoice {
    TrueColour,
    Rgb,
    Greyscale,
    Ansi,
}

impl ColEncodeChoice {
    fn arg() -> ArgExt<impl Arg<Item = Self>> {
        use ColEncodeChoice::*;
        (args_either! {
            flag("", "true-colour", "").some_if(TrueColour),
            flag("", "rgb", "").some_if(Rgb),
            flag("", "greyscale", "").some_if(Greyscale),
            flag("", "ansi", "").some_if(Ansi),
        })
        .with_default(Rgb)
    }
}

fn run<E>(mut runner: EventRoutineRunner, col_encode: E)
where
    E: ColEncode,
{
    runner
        .run(event_routine(), &mut AppData::new(), &mut AppView::new(), col_encode)
        .unwrap()
}

fn main() {
    let col_encode_choice = ColEncodeChoice::arg().with_help_default().parse_env_default_or_exit();
    let runner = Context::new().unwrap().into_runner(Duration::from_millis(16));
    use ColEncodeChoice::*;
    match col_encode_choice {
        TrueColour => run(runner, col_encode::XtermTrueColour),
        Rgb => run(runner, col_encode::FromTermInfoRgb),
        Greyscale => run(runner, col_encode::FromTermInfoGreyscale),
        Ansi => run(runner, col_encode::FromTermInfoAnsi16Colour),
    }
}
