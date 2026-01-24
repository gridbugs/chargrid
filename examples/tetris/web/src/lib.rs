use chargrid_web::{Context, Size};
use tetris_app::app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let context = Context::new(Size::new(20, 20), "content");
    context.run(app(rand::rng()));
    Ok(())
}
