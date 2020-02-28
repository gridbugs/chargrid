use prototty_ansi_terminal::{col_encode, Context};
use rand::Rng;
use rip_native::{simon::*, NativeCommon};
use rip_prototty::{app, AutoPlay, EnvNull, Frontend, RngSeed};

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
        .with_default(TrueColour)
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
    // We won't be able to print once the context is created. Choose the initial rng
    // seed before starting the game so it can be logged in case of error.
    let rng_seed_u64 = match rng_seed {
        RngSeed::U64(seed) => seed,
        RngSeed::Random => rand::thread_rng().gen(),
    };
    if let ColEncodeChoice::TrueColour = col_encode_choice {
        println!("Running in true-colour mode.\nIf colours look wrong, run with `--rgb` or try a different terminal emulator.");
    }
    println!("Initial RNG Seed: {}", rng_seed_u64);
    let context = Context::new().unwrap();
    let app = app(
        game_config,
        Frontend::AnsiTerminal,
        controls,
        file_storage,
        save_file,
        audio_player,
        RngSeed::U64(rng_seed_u64),
        Some(AutoPlay),
        None,
        Box::new(EnvNull),
    );
    use ColEncodeChoice as C;
    match col_encode_choice {
        C::TrueColour => context.run_app(app, col_encode::XtermTrueColour),
        C::Rgb => context.run_app(app, col_encode::FromTermInfoRgb),
        C::Greyscale => context.run_app(app, col_encode::FromTermInfoGreyscale),
        C::Ansi => context.run_app(app, col_encode::FromTermInfoAnsi16Colour),
    }
}
