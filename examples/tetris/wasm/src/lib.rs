extern crate prototty_wasm;
extern crate rand;
extern crate tetris_prototty;
extern crate wasm_bindgen;

use prototty_wasm::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;
use tetris_prototty::*;
use wasm_bindgen::prelude::*;

pub use prototty_wasm::InputBuffer;

#[wasm_bindgen]
pub struct WebApp {
    rng: StdRng,
    app_view: AppView,
    app: App,
    js_grid: JsGrid,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, js_grid: JsGrid) -> Self {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        let app = App::new(&mut rng);
        let app_view = AppView::new();
        Self {
            rng,
            app_view,
            app,
            js_grid,
        }
    }

    pub fn tick(&mut self, input_buffer: &InputBuffer, period_ms: f64) {
        let period = Duration::from_millis(period_ms as u64);
        self.app
            .tick(input_buffer.iter(), period, &self.app_view, &mut self.rng);
        self.js_grid.render(&mut self.app_view, &self.app);
    }
}
