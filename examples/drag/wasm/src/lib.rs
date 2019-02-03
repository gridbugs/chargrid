extern crate drag_prototty;
extern crate prototty_wasm;
extern crate wasm_bindgen;

use drag_prototty::*;
use prototty_wasm::*;
use wasm_bindgen::prelude::*;

pub use prototty_wasm::InputBuffer;

#[wasm_bindgen]
pub struct WebApp {
    app_view: AppView,
    app: App,
    js_grid: JsGrid,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(js_grid: JsGrid) -> Self {
        let app = App::default();
        let app_view = AppView;
        Self {
            app_view,
            app,
            js_grid,
        }
    }

    pub fn tick(&mut self, input_buffer: &InputBuffer) {
        self.app.update(input_buffer.iter());
        self.js_grid.render(&mut self.app_view, &self.app);
    }
}
