use prototty_web::{Context, LocalStorage, Size};
use roguelike_prototty::{event_routine, AppData, AppView, Controls, Frontend, RngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    Context::new(Size::new(24, 20), "content").run_event_routine_repeating(
        event_routine(),
        AppData::new(
            Frontend::Wasm,
            Controls::default(),
            LocalStorage::new(),
            SAVE_KEY.to_string(),
            RngSeed::Entropy,
        ),
        AppView::new(),
        |_| event_routine(),
    );
    Ok(())
}
