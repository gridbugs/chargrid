extern crate prototty_wasm;
extern crate rand;
extern crate tetris_prototty;

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;

use prototty_wasm::prototty_input::Input as ProtottyInput;
use prototty_wasm::prototty_render::Renderer;
use prototty_wasm::*;

use tetris_prototty::{App, AppView, ControlFlow};

pub struct WebApp {
    app: App,
    app_view: AppView,
    rng: StdRng,
    context: Context,
}

impl WebApp {
    fn new(seed: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        let app = App::new(&mut rng);
        let context = Context::new();
        let app_view = AppView::new();

        Self {
            app,
            app_view,
            rng,
            context,
        }
    }
    fn tick<I>(&mut self, inputs: I, period: Duration)
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        if let Some(control_flow) = self.app.tick(inputs, period, &self.app_view, &mut self.rng) {
            match control_flow {
                ControlFlow::Exit => {
                    self.context.quit();
                    return;
                }
            }
        }
        self.context.render(&mut self.app_view, &self.app).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn alloc_app(seed: usize) -> *mut WebApp {
    alloc::into_boxed_raw(WebApp::new(seed))
}

#[no_mangle]
pub unsafe fn tick(app: *mut WebApp, inputs: *const u64, num_inputs: usize, period_millis: f64) {
    let period = Duration::from_millis(period_millis as u64);

    let input_iter = input::js_event_input_iter(inputs, num_inputs);
    (*app).tick(input_iter, period);
}
