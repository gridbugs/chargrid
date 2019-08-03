extern crate prototty_event_routine;
extern crate prototty_input;
extern crate prototty_render;
extern crate prototty_text;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod event_routine;
pub use event_routine::*;

mod view;
pub use view::*;

mod instance;
pub use instance::*;
