use drag_prototty::{App, AppView, Quit};
use prototty_graphical::*;
use prototty_render::*;

fn main() {
    let size = Size::new(640, 400);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_font_scale(8.0, 8.0)
        .with_cell_dimensions(Size::new(8, 8))
        .with_max_grid_size(Size::new(120, 80))
        .build()
        .unwrap();
    let mut app = App::default();
    let mut input_buffer = Vec::with_capacity(64);
    let mut app_view = AppView;
    loop {
        let mut frame = context.frame();
        app_view.view(&app, frame.default_context(), &mut frame);
        frame.render().unwrap();
        context.buffer_input(&mut input_buffer);
        if let Some(Quit) = app.update(input_buffer.drain(..)) {
            break;
        }
    }
}
