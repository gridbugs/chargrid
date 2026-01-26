use chargrid_web::{Context, UCoord};
use colour_grid_app::app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(UCoord::new(100, 60), "content");
    context.run(app());
    Ok(())
}
