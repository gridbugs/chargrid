use colour_picker_prototty as app;
use prototty_graphical as pg;

const WINDOW_SIZE_PIXELS: pg::Size = pg::Size::new_u16(640, 480);

fn main() {
    let mut context = pg::ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(pg::Size::new_u16(16, 16))
        .build()
        .unwrap();
    context
        .run_event_routine(app::test(), &mut app::AppData::new(), &mut app::AppView::new())
        .unwrap();
}
