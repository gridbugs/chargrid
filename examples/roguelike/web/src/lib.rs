use prototty_storage::Storage;
use prototty_web::{Context, LocalStorage, Size, WebAudioPlayer};
use rip_prototty::{app, Controls, Frontend, GameConfig, RngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let audio_player = WebAudioPlayer::new_with_mime("video/ogg");
    let mut storage = LocalStorage::new();
    storage.clear();
    let context = Context::new(Size::new(40, 40), "content");
    let app = app(
        GameConfig { omniscient: None },
        Frontend::Web,
        Controls::default(),
        storage,
        SAVE_KEY.to_string(),
        audio_player,
        RngSeed::Random,
    );
    context.run_app(app);
    Ok(())
}
