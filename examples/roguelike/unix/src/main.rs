use prototty_file_storage::FileStorage;
use prototty_unix::{col_encode, Context};
use roguelike_prototty::{event_routine, AppData, AppView, Controls, Frontend, RngSeed};
use std::time::Duration;

const SAVE_DIR: &'static str = "save";

fn main() {
    let storage = FileStorage::next_to_exe(SAVE_DIR, true).expect("Failed to open save dir");
    Context::new()
        .unwrap()
        .into_runner(Duration::from_millis(16))
        .run(
            event_routine(),
            &mut AppData::new(Frontend::Native, Controls::default(), storage, RngSeed::Entropy),
            &mut AppView::new(),
            col_encode::FromTermInfoRgb,
        )
        .unwrap()
}
