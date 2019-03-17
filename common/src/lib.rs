extern crate prototty_input;
extern crate prototty_render;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod defaults;
mod pager;
mod style;
mod text;
mod text_info;
/*
mod alignment;
mod border;
mod rich_text;
mod menu;
mod string_view;
*/

pub use pager::*;
pub use style::*;
pub use text::*;
pub use text_info::*;
/*
pub use alignment::*;
pub use border::*;
pub use rich_text::*;
pub use menu::*;
pub use string_view::*;
 */
