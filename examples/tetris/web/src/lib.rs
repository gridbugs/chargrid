use prototty_web::{Context, Size};
use rand::SeedableRng;
use rand_isaac::IsaacRng;
use tetris_prototty::TetrisApp;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(20, 20), "content");
    let app = TetrisApp::new(IsaacRng::from_entropy());
    context.run_app(app);
    Ok(())
}
