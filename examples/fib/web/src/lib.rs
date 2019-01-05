extern crate fib;
extern crate prototty_wasm_storage;
extern crate wasm_bindgen;

use fib::App;
use prototty_wasm_storage::{JsByteStorage, WasmStorage};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebApp {
    app: App<WasmStorage>,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(js_byte_storage: JsByteStorage) -> Self {
        let app = App::new(WasmStorage::new(js_byte_storage));
        Self { app }
    }

    pub fn run(&mut self) -> u32 {
        let value = self.app.get();
        self.app.next_and_save();
        value
    }
}
