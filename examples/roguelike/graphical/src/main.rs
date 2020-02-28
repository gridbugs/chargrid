#![windows_subsystem = "windows"]
#[cfg(feature = "prototty_graphical")]
use prototty_graphical as graphical;
#[cfg(feature = "prototty_graphical_gfx")]
use prototty_graphical_gfx as graphical;
use rip_native::{simon::*, NativeCommon};
use rip_prototty::{app, AutoPlay, Frontend, Fullscreen};

#[cfg(feature = "prototty_graphical")]
const FULLSCREEN_SUPPORTED: bool = true;

#[cfg(feature = "prototty_graphical_gfx")]
const FULLSCREEN_SUPPORTED: bool = false;

const CELL_SIZE: f64 = 16.;

#[cfg(target_os = "windows")]
mod graphical_env {
    use rip_prototty::Env;
    use super::graphical::WindowHandle;
    use std::cell::RefCell;
    pub struct GraphicalEnv {
        window_handle: WindowHandle,
        shadow_fullscreen: RefCell<bool>,
    }
    impl GraphicalEnv {
        pub fn new(window_handle: WindowHandle) -> Self {
            Self {
                window_handle,
                shadow_fullscreen: RefCell::new(false),
            }
        }
    }
    impl Env for GraphicalEnv {
        fn fullscreen(&self) -> bool {
            *self.shadow_fullscreen.borrow()
        }
        fn fullscreen_requires_restart(&self) -> bool {
            true
        }
        fn fullscreen_supported(&self) -> bool {
            super::FULLSCREEN_SUPPORTED
        }
        fn set_fullscreen(&self, fullscreen: bool) {
            *self.shadow_fullscreen.borrow_mut() = fullscreen;
        }
        fn set_fullscreen_init(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen);
            *self.shadow_fullscreen.borrow_mut() = fullscreen;
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod graphical_env {
    use rip_prototty::Env;
    use super::graphical::WindowHandle;
    pub struct GraphicalEnv {
        window_handle: WindowHandle,
    }
    impl GraphicalEnv {
        pub fn new(window_handle: WindowHandle) -> Self {
            Self { window_handle }
        }
    }
    impl Env for GraphicalEnv {
        fn fullscreen(&self) -> bool {
            self.window_handle.fullscreen()
        }
        fn fullscreen_requires_restart(&self) -> bool {
            false
        }
        fn fullscreen_supported(&self) -> bool {
            super::FULLSCREEN_SUPPORTED
        }
        fn set_fullscreen(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen)
        }
        fn set_fullscreen_init(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen)
        }
    }
}

use graphical_env::*;
use graphical::*;

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
    let env = GraphicalEnv::new(window_handle);
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
