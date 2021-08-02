use chargrid_web::{Context, Size};
use component_experiment_app::app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(20, 20), "content");
    context.run(app());
    Ok(())
}
