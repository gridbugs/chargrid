use prototty_graphical::{ContextBuilder, Size};
use roguelike_native::{simon::Arg, NativeCommon};
use roguelike_prototty::{event_routine, AppData, AppView, Frontend};

const WINDOW_SIZE_PIXELS: Size = Size::new_u16(640, 480);

fn main() {
    env_logger::init();
    let NativeCommon {
        rng_seed,
        file_storage,
        controls,
        save_file,
    } = NativeCommon::arg().with_help_default().parse_env_or_exit();
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_min_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_max_window_dimensions(WINDOW_SIZE_PIXELS)
        .with_font_scale(14.0, 14.0)
        .with_cell_dimensions(Size::new_u16(14, 14))
        .with_underline_width(2)
        .with_underline_position(12)
        .build()
        .unwrap();
    context
        .run_event_routine(
            event_routine(),
            &mut AppData::new(Frontend::Native, controls, file_storage, save_file, rng_seed),
            &mut AppView::new(),
        )
        .unwrap();
}
