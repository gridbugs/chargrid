extern crate rand;
extern crate prototty;
extern crate prototty_unix;
extern crate prototty_app;

use std::time::Duration;
use std::thread;
use prototty::Renderer;
use prototty_unix::Context;
use prototty_app::{App, AppView, ControlFlow};

const TICK_MILLIS: u64 = 16;

fn main() {
    let mut context = Context::new().unwrap();
    let mut rng = rand::thread_rng();
    let mut app = App::new(&mut rng);

    loop {
        context.render(&AppView, &app).unwrap();
        thread::sleep(Duration::from_millis(TICK_MILLIS));

        if let Some(control_flow) = app.tick(context.drain_input().unwrap(), Duration::from_millis(TICK_MILLIS), &mut rng) {
            match control_flow {
                ControlFlow::Exit => break,
            }
        }
    }
}
