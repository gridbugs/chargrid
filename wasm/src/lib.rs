extern crate grid_2d;
extern crate js_sys;
pub extern crate prototty_event_routine;
extern crate prototty_grid;
pub extern crate prototty_input;
#[cfg(feature = "storage")]
extern crate prototty_monolithic_storage;
pub extern crate prototty_render;
#[cfg(feature = "storage")]
extern crate prototty_storage;
#[cfg(feature = "storage")]
extern crate serde;
extern crate wasm_bindgen;
extern crate web_sys;

#[cfg(feature = "storage")]
mod storage;
#[cfg(feature = "storage")]
pub use storage::*;

mod input;

use grid_2d::Coord;
pub use grid_2d::Size;
use js_sys::Function;
use prototty_grid::ColourConversion;
pub use prototty_input::Input;
use prototty_input::{MouseButton, ScrollDirection};
use prototty_render::{Rgb24, View, ViewContext, ViewContextDefault, ViewTransformRgb24};
use std::cell::RefCell;
use std::rc::Rc;
pub use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, KeyboardEvent, MouseEvent, Node, WheelEvent};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

struct WebColourConversion;
impl prototty_grid::ColourConversion for WebColourConversion {
    type Colour = Rgb24;
    fn convert_foreground_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        rgb24
    }
    fn convert_background_rgb24(&mut self, rgb24: Rgb24) -> Self::Colour {
        rgb24
    }
    fn default_foreground(&mut self) -> Self::Colour {
        Rgb24::new(255, 255, 255)
    }
    fn default_background(&mut self) -> Self::Colour {
        Rgb24::new(0, 0, 0)
    }
}

fn rgb24_to_web_colour(Rgb24 { r, g, b }: Rgb24) -> String {
    format!("rgb({},{},{})", r, g, b)
}

struct ElementCell {
    element: HtmlElement,
    character: char,
    bold: bool,
    underline: bool,
    foreground_colour: Rgb24,
    background_colour: Rgb24,
}

impl ElementCell {
    fn with_element(element: HtmlElement) -> Self {
        element.set_inner_html("&nbsp;");
        let element_style = element.style();
        element_style.set_property("color", "rgb(255,255,255)").unwrap();
        element_style.set_property("background-color", "rgb(0,0,0)").unwrap();
        element_style.set_property("display", "inline-block").unwrap();
        Self {
            element,
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: WebColourConversion.default_foreground(),
            background_colour: WebColourConversion.default_background(),
        }
    }
}

#[derive(Debug)]
struct ElementDisplayInfo {
    container_x: f64,
    container_y: f64,
    cell_width: f64,
    cell_height: f64,
}

impl ElementDisplayInfo {
    fn mouse_coord(&self, x: i32, y: i32) -> Coord {
        let x = (x - self.container_x as i32) / self.cell_width as i32;
        let y = (y - self.container_y as i32) / self.cell_height as i32;
        Coord::new(x, y)
    }
}

pub struct Context {
    element_grid: grid_2d::Grid<ElementCell>,
    prototty_grid: prototty_grid::Grid<WebColourConversion>,
    container_element: Element,
}

impl Context {
    fn element_display_info(&self) -> ElementDisplayInfo {
        let container_rect = self.container_element.get_bounding_client_rect();
        let (container_x, container_y) = (container_rect.x(), container_rect.y());
        let cell_element = self.element_grid.get_index(0).element.dyn_ref::<Element>().unwrap();
        let cell_rect = cell_element.get_bounding_client_rect();
        let (cell_width, cell_height) = (cell_rect.width(), cell_rect.height());
        ElementDisplayInfo {
            container_x,
            container_y,
            cell_width,
            cell_height,
        }
    }
    pub fn new(size: Size, container: &str) -> Self {
        if size.width() == 0 || size.height() == 0 {
            panic!("Size must not be zero");
        }
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container_node = document
            .get_element_by_id(container)
            .unwrap()
            .dyn_into::<Node>()
            .unwrap();
        let element_grid = grid_2d::Grid::new_fn(size, |_| {
            let element = document
                .create_element("span")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            ElementCell::with_element(element)
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
        Self {
            element_grid,
            prototty_grid,
            container_element: document.get_element_by_id(container).unwrap(),
        }
    }

    pub fn default_context(&self) -> ViewContextDefault {
        ViewContext::default_with_size(self.prototty_grid.size())
    }

    fn render_internal(&mut self) {
        for (prototty_cell, element_cell) in self.prototty_grid.iter().zip(self.element_grid.iter_mut()) {
            if element_cell.character != prototty_cell.character {
                element_cell.character = prototty_cell.character;
                let string = match prototty_cell.character {
                    ' ' => "&nbsp;".to_string(),
                    other => other.to_string(),
                };
                element_cell.element.set_inner_html(&string);
            }
            let element_style = element_cell.element.style();
            if element_cell.foreground_colour != prototty_cell.foreground_colour {
                element_cell.foreground_colour = prototty_cell.foreground_colour;
                element_style
                    .set_property("color", &rgb24_to_web_colour(prototty_cell.foreground_colour))
                    .unwrap();
            }
            if element_cell.background_colour != prototty_cell.background_colour {
                element_cell.background_colour = prototty_cell.background_colour;
                element_style
                    .set_property(
                        "background-color",
                        &rgb24_to_web_colour(prototty_cell.background_colour),
                    )
                    .unwrap();
            }
            if element_cell.underline != prototty_cell.underline {
                element_cell.underline = prototty_cell.underline;
                if prototty_cell.underline {
                    element_style.set_property("text-decoration", "underline").unwrap();
                } else {
                    element_style.remove_property("text-decoration").unwrap();
                }
            }
            if element_cell.bold != prototty_cell.bold {
                element_cell.bold = prototty_cell.bold;
                if prototty_cell.bold {
                    element_style.set_property("font-weight", "bold").unwrap();
                } else {
                    element_style.remove_property("font-weight").unwrap();
                }
            }
        }
    }

    pub fn render_at<V: View<T>, T, R: ViewTransformRgb24>(&mut self, view: &mut V, data: T, context: ViewContext<R>) {
        self.prototty_grid.clear();
        view.view(data, context, &mut self.prototty_grid);
        self.render_internal();
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: T) {
        let context = self.default_context();
        self.render_at(view, data, context);
    }
}

pub trait EventHandler {
    fn on_input(&mut self, input: Input, context: &mut Context);
    fn on_frame(&mut self, since_last_frame: Duration, context: &mut Context);
}

fn run_frame_handler<E: EventHandler + 'static>(event_handler: Rc<RefCell<E>>, context: Rc<RefCell<Context>>) {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let f: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut last_frame_time_stamp = performance.now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let frame_time_stamp = performance.now();
        let since_last_frame = frame_time_stamp - last_frame_time_stamp;
        last_frame_time_stamp = frame_time_stamp;
        event_handler.borrow_mut().on_frame(
            Duration::from_millis(since_last_frame as u64),
            &mut *context.borrow_mut(),
        );
        window
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));
    g.borrow()
        .as_ref()
        .unwrap()
        .as_ref()
        .unchecked_ref::<Function>()
        .call0(&JsValue::NULL)
        .unwrap();
}

mod buttons {
    pub fn has_left(buttons: u16) -> bool {
        buttons & 1 != 0
    }
    pub fn has_right(buttons: u16) -> bool {
        buttons & 2 != 0
    }
    pub fn has_middle(buttons: u16) -> bool {
        buttons & 4 != 0
    }
    pub fn has_none(buttons: u16) -> bool {
        buttons == 0
    }
}

mod button {
    use prototty_input::MouseButton;
    const LEFT: i16 = 0;
    const MIDDLE: i16 = 1;
    const RIGHT: i16 = 2;
    pub fn to_mouse_button(button: i16) -> Option<MouseButton> {
        match button {
            LEFT => Some(MouseButton::Left),
            MIDDLE => Some(MouseButton::Middle),
            RIGHT => Some(MouseButton::Right),
            _ => None,
        }
    }
}

fn run_input_handler<E: EventHandler + 'static>(event_handler: Rc<RefCell<E>>, context: Rc<RefCell<Context>>) {
    let window = web_sys::window().unwrap();
    let handle_keydown = {
        let event_handler = event_handler.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let keyboard_event = event.unchecked_ref::<KeyboardEvent>();
            if let Some(input) =
                input::from_js_event_key_press(keyboard_event.key_code() as u8, keyboard_event.shift_key())
            {
                event_handler.borrow_mut().on_input(input, &mut *context.borrow_mut());
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_move = {
        let event_handler = event_handler.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut event_handler = event_handler.borrow_mut();
            let mut context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let buttons = mouse_event.buttons();
            if buttons::has_none(buttons) {
                event_handler.on_input(Input::MouseMove { button: None, coord }, &mut *context);
            }
            if buttons::has_left(buttons) {
                event_handler.on_input(
                    Input::MouseMove {
                        button: Some(MouseButton::Left),
                        coord,
                    },
                    &mut *context,
                );
            }
            if buttons::has_right(buttons) {
                event_handler.on_input(
                    Input::MouseMove {
                        button: Some(MouseButton::Right),
                        coord,
                    },
                    &mut *context,
                );
            }
            if buttons::has_middle(buttons) {
                event_handler.on_input(
                    Input::MouseMove {
                        button: Some(MouseButton::Middle),
                        coord,
                    },
                    &mut *context,
                );
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_down = {
        let event_handler = event_handler.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut event_handler = event_handler.borrow_mut();
            let mut context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let button = mouse_event.button();
            if let Some(button) = button::to_mouse_button(button) {
                event_handler.on_input(Input::MousePress { button, coord }, &mut *context);
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_up = {
        let event_handler = event_handler.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut event_handler = event_handler.borrow_mut();
            let mut context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let button = mouse_event.button();
            if let Some(button) = button::to_mouse_button(button) {
                event_handler.on_input(
                    Input::MouseRelease {
                        button: Ok(button),
                        coord,
                    },
                    &mut *context,
                );
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_wheel = Closure::wrap(Box::new(move |event: JsValue| {
        let mut context = context.borrow_mut();
        let mut event_handler = event_handler.borrow_mut();
        let element_display_info = context.element_display_info();
        let wheel_event = event.unchecked_ref::<WheelEvent>();
        let coord = element_display_info.mouse_coord(wheel_event.client_x(), wheel_event.client_y());
        if wheel_event.delta_x() < 0. {
            event_handler.on_input(
                Input::MouseScroll {
                    direction: ScrollDirection::Left,
                    coord,
                },
                &mut *context,
            );
        } else if wheel_event.delta_x() > 0. {
            event_handler.on_input(
                Input::MouseScroll {
                    direction: ScrollDirection::Right,
                    coord,
                },
                &mut *context,
            );
        }
        if wheel_event.delta_y() < 0. {
            event_handler.on_input(
                Input::MouseScroll {
                    direction: ScrollDirection::Up,
                    coord,
                },
                &mut *context,
            );
        } else if wheel_event.delta_y() > 0. {
            event_handler.on_input(
                Input::MouseScroll {
                    direction: ScrollDirection::Down,
                    coord,
                },
                &mut *context,
            );
        }
    }) as Box<dyn FnMut(JsValue)>);
    window
        .add_event_listener_with_callback("keydown", handle_keydown.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("mousemove", handle_mouse_move.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("mousedown", handle_mouse_down.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("mouseup", handle_mouse_up.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("wheel", handle_wheel.as_ref().unchecked_ref())
        .unwrap();
    handle_keydown.forget();
    handle_mouse_move.forget();
    handle_mouse_down.forget();
    handle_mouse_up.forget();
    handle_wheel.forget();
}

pub fn run_event_handler<E: EventHandler + 'static>(event_handler: E, context: Context) {
    let event_handler = Rc::new(RefCell::new(event_handler));
    let context = Rc::new(RefCell::new(context));
    run_frame_handler(event_handler.clone(), context.clone());
    run_input_handler(event_handler, context);
}
