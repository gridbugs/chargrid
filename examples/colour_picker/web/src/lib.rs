use chargrid_web::{Context, Size};
use colour_picker_app::app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(20, 20), "content");
    context.run_app(app());
    Ok(())
}
