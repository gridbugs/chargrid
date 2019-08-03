use prototty_glutin::{ContextBuilder, Size};
use roguelike_prototty::{event_routine, AppData, AppView, Controls};
use serde_yaml;
use std::env;
use std::fs::File;
use std::io::Read;
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

fn load_controls() -> Option<Controls> {
    let mut buf = Vec::new();
    let path = this_crate_path().join(CONTROL_CONFIG);
    let mut f = File::open(path).ok()?;
    f.read_to_end(&mut buf).ok()?;
    serde_yaml::from_slice(&buf).ok()
}

fn main() {
    let controls = if let Some(controls) = load_controls() {
        controls
    } else {
        eprintln!(
            "Failed to parse control config file at {:?}. Using default controls.",
            this_crate_path().join(CONTROL_CONFIG)
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
        .with_underline_width(2)
        .with_underline_position(14)
        .build()
        .unwrap();
    context
        .run_event_routine(event_routine(), &mut AppData::new(controls), &mut AppView::new())
        .unwrap();
}
