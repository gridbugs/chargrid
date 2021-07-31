use chargrid_web::{Context, Size};
use rand::SeedableRng;
use rand_isaac::IsaacRng;
use tetris_app::app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(20, 20), "content");
    context.run_component(app(IsaacRng::from_entropy()));
    Ok(())
}
