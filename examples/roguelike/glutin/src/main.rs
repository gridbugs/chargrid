use prototty_file_storage::{format, FileStorage, Storage};
use prototty_glutin::{ContextBuilder, Size};
use roguelike_prototty::{event_routine, AppData, AppView, Controls};
use std::env;
use std::path::PathBuf;

const WINDOW_SIZE_PIXELS: Size = Size::new_u16(640, 480);

fn this_crate_path() -> PathBuf {
    env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../examples/roguelike/glutin")
}

const CONTROL_CONFIG: &'static str = "controls.yaml";

fn main() {
    let storage = FileStorage::new(this_crate_path(), false).unwrap();
    let controls = if let Ok(controls) = storage.load(CONTROL_CONFIG, format::Yaml) {
        controls
    } else {
        eprintln!(
            "Failed to parse control config file at {:?}. Using default controls.",
            storage.full_path(CONTROL_CONFIG)
        );
        Controls::default()
    };
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
        .run_event_routine(event_routine(), &mut AppData::new(controls), &mut AppView::new())
        .unwrap();
}
