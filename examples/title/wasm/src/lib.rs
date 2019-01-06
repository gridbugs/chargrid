extern crate prototty_title;
extern crate prototty_wasm_render;
extern crate wasm_bindgen;

use prototty_title::*;
use prototty_wasm_render::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebApp {
    app: Title,
    view: DemoTitleView,
    js_grid: JsGrid,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(js_grid: JsGrid) -> Self {
        WebApp {
            app: Title {
                width: 20,
                text: "My Title".to_string(),
            },
            view: DemoTitleView,
            js_grid,
        }
    }

    pub fn run(&mut self, js_renderer: &JsRenderer) {
        self.js_grid.view(&mut self.view, &self.app);
        js_renderer.js_render(&self.js_grid);
    }
}
