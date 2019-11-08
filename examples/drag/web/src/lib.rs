use drag_prototty::*;
use prototty_web::*;
use wasm_bindgen::prelude::*;

struct WebApp {
    app_view: AppView,
    app: App,
    input_buffer: Vec<Input>,
}

impl EventHandler for WebApp {
    fn on_input(&mut self, input: Input, _context: &mut Context) {
        self.input_buffer.push(input);
    }
    fn on_frame(&mut self, _since_last_frame: Duration, context: &mut Context) {
        self.app.update(self.input_buffer.drain(..));
        context.render(&mut self.app_view, &self.app);
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let web_app = WebApp {
        app_view: AppView,
        app: App::default(),
        input_buffer: Vec::new(),
    };
    Context::new(Size::new(80, 40), "content").run_event_handler(web_app);
    Ok(())
}
