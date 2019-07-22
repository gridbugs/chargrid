extern crate fib;
extern crate prototty_wasm;
extern crate wasm_bindgen;

use fib::App;
use prototty_wasm::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(js_byte_storage: JsByteStorage) -> Result<(), JsValue> {
    let mut app = App::new(WasmStorage::new(js_byte_storage));
    console_log!("{}", app.get());
    app.next_and_save();
    Ok(())
}
