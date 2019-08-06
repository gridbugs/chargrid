use prototty_unix::{col_encode, Context};
use roguelike_native::NativeCommon;
use roguelike_prototty::{event_routine, AppData, AppView, Frontend};
use std::time::Duration;

fn main() {
    let NativeCommon {
        rng_seed,
        file_storage,
        controls,
        save_file,
    } = NativeCommon::arg().with_help_default().parse_env_default_or_exit();
    Context::new()
        .unwrap()
        .into_runner(Duration::from_millis(16))
        .run(
            event_routine(),
            &mut AppData::new(Frontend::Native, controls, file_storage, save_file, rng_seed),
            &mut AppView::new(),
            col_encode::FromTermInfoRgb,
        )
        .unwrap()
}
