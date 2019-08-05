use prototty_wasm::{Context, JsByteStorage, Size, WasmStorage};
use roguelike_prototty::{event_routine, AppData, AppView, Controls, Frontend, RngSeed};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(js_byte_storage: JsByteStorage) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let storage = WasmStorage::new(js_byte_storage);
    Context::new(Size::new(20, 20), "content").run_event_routine_repeating(
        event_routine(),
        AppData::new(Frontend::Wasm, Controls::default(), storage, RngSeed::Entropy),
        AppView::new(),
        |_| event_routine(),
    );
    Ok(())
}
