use pager_app::App;
use chargrid_web::{Context, Size};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let text = include_str!("../../sample.txt").to_string();
    let context = Context::new(Size::new(80, 40), "content");
    context.run_app(App::new(text));
    Ok(())
}
