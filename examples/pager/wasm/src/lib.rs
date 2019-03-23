extern crate pager_prototty;
extern crate prototty_wasm;
extern crate wasm_bindgen;

use pager_prototty::*;
use prototty_wasm::*;
use wasm_bindgen::prelude::*;

pub use prototty_wasm::InputBuffer;

#[wasm_bindgen]
pub struct WebApp {
    app_view: AppView,
    app_state: AppState,
    js_grid: JsGrid,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(js_grid: JsGrid) -> Self {
        let string = include_str!("../../sample.txt").to_string();
        let app_state = AppState::new(string);
        let app_view = AppView::new();
        Self {
            app_view,
            app_state,
            js_grid,
        }
    }

    pub fn tick(&mut self, input_buffer: &InputBuffer) {
        self.app_state.tick(input_buffer.iter(), &self.app_view);
        self.js_grid.render(&mut self.app_view, &self.app_state);
    }
}
