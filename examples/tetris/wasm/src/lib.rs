use prototty_wasm::*;
use rand::SeedableRng;
use rand_isaac::IsaacRng;
use tetris_prototty::*;
use wasm_bindgen::prelude::*;

struct WebApp {
    rng: IsaacRng,
    app_view: AppView,
    app: App,
    input_buffer: Vec<Input>,
}

impl EventHandler for WebApp {
    fn on_input(&mut self, input: Input, _context: &mut Context) {
        self.input_buffer.push(input);
    }
    fn on_frame(&mut self, since_last_frame: Duration, context: &mut Context) {
        self.app.tick(
            self.input_buffer.drain(..),
            since_last_frame,
            &self.app_view,
            &mut self.rng,
        );
        context.render(&mut self.app_view, &self.app);
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let mut rng = IsaacRng::from_entropy();
    let app = App::new(&mut rng);
    let app_view = AppView::new();
    let web_app = WebApp {
        rng,
        app_view,
        app,
        input_buffer: Vec::new(),
    };
    Context::new(Size::new(20, 20), "content").run_event_handler(web_app);
    Ok(())
}
