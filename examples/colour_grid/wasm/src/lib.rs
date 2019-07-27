use colour_grid_prototty::{event_routine, AppData, AppView};
use prototty_wasm::{Context, Size};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    Context::new(Size::new(20, 20), "content").run_event_routine_repeating(
        event_routine(),
        AppData::new(),
        AppView::new(),
        |_| event_routine(),
    );
    Ok(())
}
