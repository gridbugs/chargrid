extern crate prototty_decorator;
extern crate prototty_input;
extern crate prototty_menu;
extern crate prototty_render;
#[cfg(feature = "storage")]
extern crate prototty_storage;
extern crate prototty_text;
extern crate prototty_tick_routine;

pub use prototty_decorator::*;
pub use prototty_input::*;
pub use prototty_menu::*;
pub use prototty_render::*;
#[cfg(feature = "storage")]
pub use prototty_storage::*;
pub use prototty_text::*;

pub use prototty_input::inputs as prototty_inputs;
pub use prototty_input::Input as ProtottyInput;
pub use prototty_menu::tick_routine as menu_tick_routine;
