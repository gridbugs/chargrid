extern crate pager_prototty;
extern crate prototty_glutin;

use pager_prototty::*;
use prototty_glutin::*;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut string = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut string)?;
    let mut view = AppView::new();
    let mut state = AppState::new(string);
    let size = Size::new(640, 960);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_font_scale(12.0, 12.0)
        .with_cell_dimensions(Size::new(12, 12))
        .with_max_grid_size(Size::new(120, 200))
        .build()
        .unwrap();
    let mut input_buffer = Vec::with_capacity(64);
    loop {
        context.render(&mut view, &state).unwrap();
        context.buffer_input(&mut input_buffer);
        match state.tick(input_buffer.drain(..), &view) {
            None => (),
            Some(ControlFlow::Exit) => break,
        }
    }
    Ok(())
}
