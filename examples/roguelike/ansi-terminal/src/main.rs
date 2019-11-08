use prototty_ansi_terminal::{col_encode, ColEncode, Context, EventRoutineRunner};
use roguelike_native::{simon::*, FileStorage, NativeCommon};
use roguelike_prototty::{event_routine, AppData, AppView, Frontend};
use std::time::Duration;

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

fn run<E>(mut runner: EventRoutineRunner, mut app_data: AppData<FileStorage>, mut app_view: AppView, col_encode: E)
where
    E: ColEncode,
{
    runner
        .run(event_routine(), &mut app_data, &mut app_view, col_encode)
        .unwrap()
}

struct Args {
    native_common: NativeCommon,
    col_encode_choice: ColEncodeChoice,
}

impl Args {
    fn arg() -> impl Arg<Item = Self> {
        args_map! {
            let {
                native_common = NativeCommon::arg();
                col_encode_choice = ColEncodeChoice::arg();
            } in {
                Self { native_common, col_encode_choice }
            }
        }
    }
}

fn main() {
    let Args {
        native_common:
            NativeCommon {
                rng_seed,
                file_storage,
                controls,
                save_file,
            },
        col_encode_choice,
    } = Args::arg().with_help_default().parse_env_or_exit();
    let runner = Context::new().unwrap().into_runner(Duration::from_millis(16));
    let app_data = AppData::new(Frontend::Native, controls, file_storage, save_file, rng_seed);
    let app_view = AppView::new();
    use ColEncodeChoice::*;
    match col_encode_choice {
        TrueColour => run(runner, app_data, app_view, col_encode::XtermTrueColour),
        Rgb => run(runner, app_data, app_view, col_encode::FromTermInfoRgb),
        Greyscale => run(runner, app_data, app_view, col_encode::FromTermInfoGreyscale),
        Ansi => run(runner, app_data, app_view, col_encode::FromTermInfoAnsi16Colour),
    }
}
