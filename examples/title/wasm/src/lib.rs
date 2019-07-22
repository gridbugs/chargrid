extern crate prototty_title;
extern crate prototty_wasm;
extern crate wasm_bindgen;

use prototty_title::*;
use prototty_wasm::*;
use wasm_bindgen::prelude::*;

pub struct WebApp {
    app: Title,
    view: DemoTitleView,
}

impl EventHandler for WebApp {
    fn on_input(&mut self, _input: Input, _context: &mut Context) {}
    fn on_frame(&mut self, _since_last_frame: Duration, context: &mut Context) {
        context.render(&mut self.view, &self.app);
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let web_app = WebApp {
        app: Title {
            width: 20,
            text: "My Title".to_string(),
        },
        view: DemoTitleView,
    };
    let context = Context::new(Size::new(80, 40), "content");
    run_event_handler(web_app, context);
    Ok(())
}
