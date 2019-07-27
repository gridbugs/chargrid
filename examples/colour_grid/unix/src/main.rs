#[macro_use]
extern crate simon;

use colour_grid_prototty::{event_routine, AppData, AppView};
use prototty_unix::{encode_colour, Context, EncodeColour, EventRoutineRunner};
use std::time::Duration;

#[derive(Clone)]
enum EncodeColourChoice {
    TrueColour,
    Rgb,
    Greyscale,
    Ansi,
}

impl EncodeColourChoice {
    fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        use EncodeColourChoice::*;
        (args_either! {
            simon::flag("", "true-colour", "").some_if(TrueColour),
            simon::flag("", "rgb", "").some_if(Rgb),
            simon::flag("", "greyscale", "").some_if(Greyscale),
            simon::flag("", "ansi", "").some_if(Ansi),
        })
        .with_default(Rgb)
    }
}

fn run<E>(mut runner: EventRoutineRunner, encode_colour: E)
where
    E: EncodeColour,
{
    runner
        .run(event_routine(), &mut AppData::new(), &mut AppView::new(), encode_colour)
        .unwrap()
}

fn main() {
    let encode_colour_choice = EncodeColourChoice::arg()
        .with_help_default()
        .parse_env_default_or_exit();
    let runner = Context::new().unwrap().into_runner(Duration::from_millis(16));
    use EncodeColourChoice::*;
    match encode_colour_choice {
        TrueColour => run(runner, encode_colour::XtermTrueColour),
        Rgb => run(runner, encode_colour::FromTermInfoRgb),
        Greyscale => run(runner, encode_colour::FromTermInfoGreyscale),
        Ansi => run(runner, encode_colour::FromTermInfoAnsi16Colour),
    }
}
