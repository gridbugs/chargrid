//! A library for defining and displaying terminal user interfaces.
//!
//! A user interface consists of:
//!
//!  - a data structure encapsulating the state of the user interface
//!  - logic for rendering the data structure
//!
//! To create a user interface, define a type which represents
//! the user interface's state, and implement the `View` trait
//! for it.
//!
//! ## Basic Usage
//!
//! Here's a tiny example that renders a character in the top-left
//! corner of the terminal for half a second.
//!
//! ```
//! extern crate cgmath;
//! extern crate prototty;
//! use std::thread;
//! use std::time::Duration;
//! use cgmath::Vector2;
//! use prototty::{Context, View, ViewGrid, ViewCell};
//!
//! struct ExampleState {
//!   character: char,
//! }
//!
//! impl View for ExampleState {
//!   fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
//!     if let Some(cell) = grid.get_mut(Vector2::new(0, 0)) {
//!       cell.update(self.character, depth);
//!     }
//!   }
//! }
//!
//! fn main() {
//!   let mut ctx = Context::new().expect("Failed to initialize context");
//!   let state = ExampleState {
//!     character: 'a',
//!   };
//!   ctx.render(&state).expect("Failed to render");
//!
//!   thread::sleep(Duration::from_millis(500));
//! }
//!
//! ```
//!
//! ## Modular User Interfaces
//!
//! It's easy to construct complex user interfaces by combining simple user interface elements.
//! The `view` method defined for complex user interfaces can call the `view` methods
//! of its constituent user interface elements with appropriate offsets.
//!
//! ```
//! extern crate cgmath;
//! extern crate prototty;
//! use std::thread;
//! use std::time::Duration;
//! use cgmath::Vector2;
//! use prototty::{Context, View, ViewGrid, ViewCell};
//!
//! // Renders a single line of text
//! struct SimpleText {
//!   text: String,
//! }
//!
//! impl View for SimpleText {
//!   fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
//!     for (i, character) in self.text.chars().enumerate() {
//!       let coord = offset + Vector2::new(i, 0).cast();
//!       if let Some(cell) = grid.get_mut(coord) {
//!         cell.update(character, depth);
//!       }
//!     }
//!   }
//! }
//!
//! // Renders two lines of text
//! struct ComplexText {
//!   first: SimpleText,
//!   second: SimpleText,
//! }
//!
//! impl View for ComplexText {
//!   fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
//!     self.first.view(offset + Vector2::new(1, 1), depth, grid);
//!     self.second.view(offset + Vector2::new(1, 3), depth, grid);
//!   }
//! }
//!
//! fn main() {
//!   let mut ctx = Context::new().expect("Failed to initialize context");
//!   let state = ComplexText {
//!     first: SimpleText { text: "hello".to_string() },
//!     second: SimpleText { text: "world".to_string() },
//!   };
//!   ctx.render(&state).expect("Failed to render");
//!
//!   thread::sleep(Duration::from_millis(500));
//! }
//!
//! ```
//!
//! ## Common Elements
//!
//! Some common user interface elements are implemented in a separate crate:
//! [prototty_elements](https://crates.io/crates/prototty_elements)


extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate libc;
extern crate term;
extern crate cgmath;
extern crate ansi_colour;
#[macro_use] extern crate itertools;

mod core;
mod grid;
mod error;
mod input;
mod context;
mod defaults;
mod view;
mod iterators;

pub use self::error::{Result, Error};
pub use self::input::Input;
pub use self::context::*;
pub use self::view::*;
