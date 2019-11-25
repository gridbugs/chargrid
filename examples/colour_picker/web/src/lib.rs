use colour_picker_prototty::app;
use prototty_web::{Context, Size};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(20, 20), "content");
    context.run_app(app());
    Ok(())
}
