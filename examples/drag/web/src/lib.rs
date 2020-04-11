use drag_app::App;
use chargrid_web::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let context = Context::new(Size::new(80, 40), "content");
    context.run_app(App::default());
    Ok(())
}
