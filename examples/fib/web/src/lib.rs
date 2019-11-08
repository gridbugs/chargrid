use fib::App;
use prototty_web::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    let mut app = App::new(LocalStorage::new());
    log::info!("{}", app.get());
    app.next_and_save();
    Ok(())
}
