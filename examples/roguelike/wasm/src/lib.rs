use prototty_wasm::{Context, LocalStorage, Size};
use roguelike_prototty::{event_routine, AppData, AppView, Controls, Frontend, RngSeed};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Context::new(Size::new(20, 20), "content").run_event_routine_repeating(
        event_routine(),
        AppData::new(
            Frontend::Wasm,
            Controls::default(),
            LocalStorage::new(),
            RngSeed::Entropy,
        ),
        AppView::new(),
        |_| event_routine(),
    );
    Ok(())
}
