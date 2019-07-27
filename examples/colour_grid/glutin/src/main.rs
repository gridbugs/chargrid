use colour_grid_prototty::{event_routine, AppData, AppView};
use prototty_glutin::{ContextBuilder, Size};

const WINDOW_SIZE_PIXELS: Size = Size::new_u16(640, 480);

fn main() {
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(Size::new_u16(16, 16))
        .build()
        .unwrap();
    context
        .run_event_routine(event_routine(), &mut AppData::new(), &mut AppView::new())
        .unwrap();
}
