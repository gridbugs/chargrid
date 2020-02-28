#![windows_subsystem = "windows"]
#[cfg(feature = "prototty_graphical")]
use prototty_graphical::*;
#[cfg(feature = "prototty_graphical_gfx")]
use prototty_graphical_gfx::*;
use rip_native::{simon::*, NativeCommon};
use rip_prototty::{app, AutoPlay, Env, Frontend, Fullscreen};

const CELL_SIZE: f64 = 16.;

struct GraphicalEnv {
    window_handle: WindowHandle,
}
impl Env for GraphicalEnv {
    fn fullscreen(&self) -> bool {
        self.window_handle.fullscreen()
    }
    fn set_fullscreen(&self, fullscreen: bool) {
        self.window_handle.set_fullscreen(fullscreen)
    }
}

struct Args {
    native_common: NativeCommon,
    fullscreen: Option<Fullscreen>,
}

impl Args {
    fn arg() -> impl Arg<Item = Self> {
        args_map! {
            let {
                native_common = NativeCommon::arg();
                fullscreen = flag("", "fullscreen", "start in fullscreen").some_if(Fullscreen);
            } in {
                Self { native_common, fullscreen }
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
        fullscreen,
    } = Args::arg().with_help_default().parse_env_or_exit();
    let (context, window_handle) = Context::new_returning_window_handle(ContextDescriptor {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin-with-quadrant-blocks.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA-with-quadrant-blocks.ttf").to_vec(),
        },
        title: "RIP".to_string(),
        window_dimensions: Dimensions {
            width: 960.,
            height: 640.,
        },
        cell_dimensions: Dimensions {
            width: CELL_SIZE,
            height: CELL_SIZE,
        },
        font_dimensions: Dimensions {
            width: CELL_SIZE,
            height: CELL_SIZE,
        },
        font_source_dimensions: Dimensions {
            width: CELL_SIZE as f32,
            height: CELL_SIZE as f32,
        },
        underline_width: 0.1,
        underline_top_offset: 0.8,
    })
    .unwrap();
    let env = GraphicalEnv { window_handle };
    let app = app(
        game_config,
        Frontend::Graphical,
        controls,
        file_storage,
        save_file,
        audio_player,
        rng_seed,
        Some(AutoPlay),
        fullscreen,
        Box::new(env),
    );
    context.run_app(app);
}
