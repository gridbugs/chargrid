use prototty_wasm::{Context, Size};
use roguelike_prototty::{event_routine, AppData, AppView, Controls};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    Context::new(Size::new(20, 20), "content").run_event_routine_repeating(
        event_routine(),
        AppData::new(Controls::default()),
        AppView::new(),
        |_| event_routine(),
    );
    Ok(())
}
