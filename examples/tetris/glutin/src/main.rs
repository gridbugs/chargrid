extern crate prototty_glutin;
extern crate rand;
extern crate tetris_prototty;

use prototty_glutin::*;
use std::time::Instant;
use tetris_prototty::{App, AppView, ControlFlow};

fn main() {
    let size = Size::new(640, 400);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(Size::new(16, 16))
        .with_underline_position(16)
        .with_underline_width(4)
        .with_max_grid_size(Size::new(45, 25))
        .build()
        .unwrap();
    let mut rng = rand::thread_rng();
    let mut app = App::new(&mut rng);
    let mut input_buffer = Vec::with_capacity(64);
    let mut app_view = AppView::new();
    let mut last_instant = Instant::now();
    let mut running = true;
    loop {
        context.render(&mut app_view, &app).unwrap();
        if !running {
            break;
        }
        let now = Instant::now();
        let duration = now - last_instant;
        last_instant = now;
        context.poll_input(|input| {
            input_buffer.push(input);
        });
        if let Some(control_flow) = app.tick(input_buffer.drain(..), duration, &app_view, &mut rng) {
            match control_flow {
                ControlFlow::Exit => running = false,
            }
        }
    }
}
