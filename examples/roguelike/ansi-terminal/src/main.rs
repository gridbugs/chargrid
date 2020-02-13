use prototty_ansi_terminal::{col_encode, Context};
use rip_native::{simon::*, NativeCommon};
use rip_prototty::{app, Frontend};

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
    env_logger::init();
    let Args {
        native_common:
            NativeCommon {
                rng_seed,
                file_storage,
                controls,
                save_file,
                audio_player,
                game_config,
            },
        col_encode_choice,
    } = Args::arg().with_help_default().parse_env_or_exit();
    let context = Context::new().unwrap();
    let app = app(
        game_config,
        Frontend::Native,
        controls,
        file_storage,
        save_file,
        audio_player,
        rng_seed,
    );
    use ColEncodeChoice as C;
    match col_encode_choice {
        C::TrueColour => context.run_app(app, col_encode::XtermTrueColour),
        C::Rgb => context.run_app(app, col_encode::FromTermInfoRgb),
        C::Greyscale => context.run_app(app, col_encode::FromTermInfoGreyscale),
        C::Ansi => context.run_app(app, col_encode::FromTermInfoAnsi16Colour),
    }
}
