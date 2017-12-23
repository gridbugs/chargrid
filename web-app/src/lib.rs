extern crate tetris;
extern crate prototty;
extern crate rand;
extern crate prototty_renderer;

use std::time::Duration;
use std::slice;
use rand::SeedableRng;

pub struct App {
    tetris: tetris::Tetris,
    model: prototty_renderer::Model,
    context: prototty::Context,
    rng: rand::StdRng,
}

impl App {
    fn new(seed: usize) -> Self {
        let mut rng = rand::StdRng::from_seed(&[seed]);
        let context = prototty::Context::new().unwrap();
        let tetris = tetris::Tetris::new(&mut rng);
        let size = tetris.size();
        let model = prototty_renderer::Model::new(size.x, size.y);

        Self {
            tetris,
            model,
            context,
            rng,
        }
    }
    fn tick(&mut self) {
        self.model.render(&self.tetris);
        self.context.render(&self.model).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn alloc_app(seed: usize) -> *mut App {
    let app = Box::new(App::new(seed));
    Box::into_raw(app)
}

#[no_mangle]
pub unsafe fn tick(app: *mut App, input_buffer: *const u8, num_inputs: usize, period_millis: f64) {
    let period = Duration::from_millis(period_millis as u64);
    let inputs = slice::from_raw_parts(input_buffer, num_inputs);
    (*app).tick();
}
