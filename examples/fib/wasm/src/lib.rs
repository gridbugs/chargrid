use fib::App;
use prototty_wasm::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let mut app = App::new(LocalStorage::new());
    console_log!("{}", app.get());
    app.next_and_save();
    Ok(())
}
