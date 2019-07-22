extern crate colour_picker_prototty;
extern crate prototty_wasm;
extern crate wasm_bindgen;

use colour_picker_prototty as app;
use prototty_wasm as pw;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    pw::Context::new(pw::Size::new(20, 20), "content").run_event_routine_repeating(
        app::test(),
        app::AppData::new(),
        app::AppView::new(),
        |_| app::test(),
    );
    Ok(())
}
