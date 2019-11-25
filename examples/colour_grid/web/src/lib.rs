use colour_grid_prototty::app;
use prototty_web::{Context, Size};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(100, 60), "content");
    context.run_app(app());
    Ok(())
}
