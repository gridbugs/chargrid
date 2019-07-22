extern crate pager_prototty;
extern crate prototty_wasm2;
extern crate wasm_bindgen;

use pager_prototty::*;
use prototty_wasm2::*;
use wasm_bindgen::prelude::*;

pub struct WebApp {
    app_view: AppView,
    app_state: AppState,
    input_buffer: Vec<Input>,
}

impl EventHandler for WebApp {
    fn on_input(&mut self, input: Input, _context: &mut Context) {
        self.input_buffer.push(input);
    }
    fn on_frame(&mut self, _since_last_frame: Duration, context: &mut Context) {
        self.app_state.tick(self.input_buffer.drain(..), &self.app_view);
        context.render(&mut self.app_view, &self.app_state);
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let string = include_str!("../../sample.txt").to_string();
    let app_state = AppState::new(string);
    let app_view = AppView::new();
    let web_app = WebApp {
        app_view,
        app_state,
        input_buffer: Vec::new(),
    };
    let context = Context::new(Size::new(80, 40), "content");
    run_event_handler(web_app, context);
    Ok(())
}
