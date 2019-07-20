extern crate grid_2d;
extern crate js_sys;
extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;
extern crate wasm_bindgen;
extern crate web_sys;

use grid_2d::Coord;
pub use grid_2d::Size;
use prototty_input::Input;
use prototty_render::{Rgb24, View};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent, Node, WheelEvent};

struct WebColourConversion;
impl prototty_grid::ColourConversion for WebColourConversion {
    type Colour = String;
    fn convert_foreground_rgb24(&mut self, Rgb24 { r, g, b }: Rgb24) -> Self::Colour {
        format!("rgb({},{},{})", r, g, b)
    }
    fn convert_background_rgb24(&mut self, Rgb24 { r, g, b }: Rgb24) -> Self::Colour {
        format!("rgb({},{},{})", r, g, b)
    }
    fn default_foreground(&mut self) -> Self::Colour {
        "rgb(255,255,255)".to_string()
    }
    fn default_background(&mut self) -> Self::Colour {
        "rgb(0,0,0)".to_string()
    }
}

struct ElementCell {
    element: HtmlElement,
}

pub struct Context {
    element_grid: grid_2d::Grid<ElementCell>,
    prototty_grid: prototty_grid::Grid<WebColourConversion>,
}

impl Context {
    pub fn new(size: Size, container: &str) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container_node = document
            .get_element_by_id(container)
            .unwrap()
            .dyn_into::<Node>()
            .unwrap();
        let element_grid = grid_2d::Grid::new_fn(size, |coord| {
            let element = document
                .create_element("span")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            element.set_inner_text(".");
            ElementCell { element }
        });
        for y in 0..size.height() {
            for x in 0..size.width() {
                container_node
                    .append_child(&element_grid.get_checked(Coord::new(x as i32, y as i32)).element)
                    .unwrap();
            }
            container_node
                .append_child(document.create_element("br").unwrap().dyn_ref::<HtmlElement>().unwrap())
                .unwrap();
        }
        let prototty_grid = prototty_grid::Grid::new(size, WebColourConversion);
        let style_text = format!(
            "#{} {{
                font-family: monospace;
                font-size: 24px;
            }}",
            container
        );
        let style_element = document
            .create_element("style")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        style_element.set_inner_text(&style_text);
        document.head().unwrap().append_child(&style_element).unwrap();
        Self {
            element_grid,
            prototty_grid,
        }
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: T) {}
}

pub trait EventHandler {
    fn on_input(&mut self, input: Input, context: &mut Context);
    fn on_frame(&mut self, since_last_frame: Duration, context: &mut Context);
}

pub fn run_event_handler<E: EventHandler>(event_handler: &mut E, context: &mut Context) {}
