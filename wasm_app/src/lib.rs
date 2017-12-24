extern crate tetris;
extern crate prototty;
extern crate rand;
extern crate prototty_app;

use std::mem;
use std::slice;
use std::time::Duration;
use rand::{SeedableRng, StdRng};
use prototty::wasm::*;
use prototty::traits::Renderer;
use prototty::input::{self as prototty_input, Input as ProtottyInput};

use prototty_app::{App, AppView};

pub struct WebApp {
    app: App,
    rng: StdRng,
    context: Context,
}

impl WebApp {
    fn new(seed: usize) -> Self {
        let mut rng = rand::StdRng::from_seed(&[seed]);
        let app = App::new(&mut rng);
        let context = Context::new();

        Self {
            app,
            rng,
            context,
        }
    }
    fn tick<I>(&mut self, inputs: I, period: Duration)
        where I: Iterator<Item=ProtottyInput>,
    {
        self.app.tick(inputs, period, &mut self.rng);
        self.context.render(&AppView, &self.app).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn alloc_app(seed: usize) -> *mut WebApp {
    let app = Box::new(WebApp::new(seed));
    Box::into_raw(app)
}

#[no_mangle]
pub extern "C" fn alloc_buf(size: usize) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe fn tick(app: *mut WebApp, input_buffer: *const u8, num_inputs: usize, period_millis: f64) {
    let period = Duration::from_millis(period_millis as u64);
    let inputs = slice::from_raw_parts(input_buffer, num_inputs);
    let prototty_input_iter = inputs.iter().filter_map(|i| {
        match i {
            &37 => Some(ProtottyInput::Left),
            &38 => Some(ProtottyInput::Up),
            &39 => Some(ProtottyInput::Right),
            &40 => Some(ProtottyInput::Down),
            &27 => Some(prototty_input::ESCAPE),
            &13 => Some(prototty_input::RETURN),
            _ => None,
        }
    });
    (*app).tick(prototty_input_iter, period);
}
