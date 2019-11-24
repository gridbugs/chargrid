use prototty_storage::Storage;
use prototty_web::{Context, LocalStorage, Size, WebAudioPlayer};
use roguelike_prototty::{event_routine, AppData, AppView, Controls, Frontend, RngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let audio_player = WebAudioPlayer::new_with_mime("video/ogg");
    let mut storage = LocalStorage::new();
    storage.clear();
    Context::new(Size::new(40, 40), "content").run_event_routine_one_shot_ignore_return(
        event_routine(),
        AppData::new(
            Frontend::Wasm,
            Controls::default(),
            storage,
            SAVE_KEY.to_string(),
            audio_player,
            RngSeed::Entropy,
        ),
        AppView::new(),
    );
    Ok(())
}
