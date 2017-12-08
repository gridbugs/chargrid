extern crate prototty;
extern crate cgmath;
extern crate ansi_colour;
#[macro_use] extern crate itertools;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod defaults;

pub mod common;

/// Borders around other elements
pub mod border;

/// Drawing canvas
pub mod canvas;

/// Graphical menus
pub mod menu;

/// Rich text
pub mod rich_text;

/// Plain text
pub mod text;

pub mod elements {
    //! Re-exports all elements

    pub use super::border::Border;
    pub use super::canvas::Canvas;
    pub use super::menu::MenuInstance;
    pub use super::rich_text::RichText;
    pub use super::text::Text;
}
