use general_audio_web::WebAudioPlayer;
use general_storage_web::LocalStorage;
use prototty_web::{Context, Size};
use rip_prototty::{app, AutoPlay, Controls, EnvNull, Frontend, GameConfig, RngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let audio_player = WebAudioPlayer::new_with_mime("video/ogg");
    let storage = LocalStorage::new();
    let context = Context::new(Size::new(60, 40), "content");
    let app = app(
        GameConfig { omniscient: None },
        Frontend::Web,
        Controls::default(),
        storage,
        SAVE_KEY.to_string(),
        audio_player,
        RngSeed::Random,
        Some(AutoPlay),
        None,
        Box::new(EnvNull),
    );
    context.run_app(app);
    Ok(())
}
