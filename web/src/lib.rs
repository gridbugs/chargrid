mod input;

use grid_2d::Coord;
pub use grid_2d::Size;
use js_sys::Function;
use prototty_app::App;
pub use prototty_input;
pub use prototty_input::{Input, MouseInput};
use prototty_input::{MouseButton, ScrollDirection};
pub use prototty_render;
use prototty_render::{Buffer, Rgb24, ViewContext};
use std::cell::RefCell;
use std::rc::Rc;
pub use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, KeyboardEvent, MouseEvent, Node, WheelEvent};

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
        Self {
            element,
            character: ' ',
            bold: false,
            underline: false,
            foreground_colour: Rgb24::new_grey(0),
            background_colour: Rgb24::new_grey(0),
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
    buffer: Buffer,
    container_element: Element,
}

impl Context {
    fn element_display_info(&self) -> ElementDisplayInfo {
        let container_rect = self.container_element.get_bounding_client_rect();
        let (container_x, container_y) = (container_rect.x(), container_rect.y());
        let cell_element = self
            .element_grid
            .get_index_checked(0)
            .element
            .dyn_ref::<Element>()
            .unwrap();
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
        let buffer = Buffer::new(size);
        Self {
            element_grid,
            buffer,
            container_element: document.get_element_by_id(container).unwrap(),
        }
    }

    fn render_internal(&mut self) {
        for (prototty_cell, element_cell) in self.buffer.iter().zip(self.element_grid.iter_mut()) {
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

    pub fn run_app<A>(self, app: A)
    where
        A: App + 'static,
    {
        let app = Rc::new(RefCell::new(app));
        let context = Rc::new(RefCell::new(self));
        run_app_frame(app.clone(), context.clone());
        run_app_input(app, context);
    }
}

fn run_app_frame<A: App + 'static>(app: Rc<RefCell<A>>, context: Rc<RefCell<Context>>) {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let f: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut last_frame_time_stamp = performance.now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let frame_time_stamp = performance.now();
        let since_last_frame = frame_time_stamp - last_frame_time_stamp;
        last_frame_time_stamp = frame_time_stamp;
        let mut context = context.borrow_mut();
        context.buffer.clear();
        let view_context = ViewContext::default_with_size(context.buffer.size());
        app.borrow_mut().on_frame(
            Duration::from_millis(since_last_frame as u64),
            view_context,
            &mut context.buffer,
        );
        context.render_internal();
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

fn run_app_input<A: App + 'static>(app: Rc<RefCell<A>>, context: Rc<RefCell<Context>>) {
    let window = web_sys::window().unwrap();
    let handle_keydown = {
        let app = app.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let keyboard_event = event.unchecked_ref::<KeyboardEvent>();
            if let Some(input) =
                input::from_js_event_key_press(keyboard_event.key_code() as u8, keyboard_event.shift_key())
            {
                app.borrow_mut().on_input(input);
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_move = {
        let app = app.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut app = app.borrow_mut();
            let context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let buttons = mouse_event.buttons();
            if buttons::has_none(buttons) {
                app.on_input(Input::Mouse(MouseInput::MouseMove { button: None, coord }));
            }
            if buttons::has_left(buttons) {
                app.on_input(Input::Mouse(MouseInput::MouseMove {
                    button: Some(MouseButton::Left),
                    coord,
                }));
            }
            if buttons::has_right(buttons) {
                app.on_input(Input::Mouse(MouseInput::MouseMove {
                    button: Some(MouseButton::Right),
                    coord,
                }));
            }
            if buttons::has_middle(buttons) {
                app.on_input(Input::Mouse(MouseInput::MouseMove {
                    button: Some(MouseButton::Middle),
                    coord,
                }));
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_down = {
        let app = app.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut app = app.borrow_mut();
            let context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let button = mouse_event.button();
            if let Some(button) = button::to_mouse_button(button) {
                app.on_input(Input::Mouse(MouseInput::MousePress { button, coord }));
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_mouse_up = {
        let app = app.clone();
        let context = context.clone();
        Closure::wrap(Box::new(move |event: JsValue| {
            let mut app = app.borrow_mut();
            let context = context.borrow_mut();
            let element_display_info = context.element_display_info();
            let mouse_event = event.unchecked_ref::<MouseEvent>();
            let coord = element_display_info.mouse_coord(mouse_event.client_x(), mouse_event.client_y());
            let button = mouse_event.button();
            if let Some(button) = button::to_mouse_button(button) {
                app.on_input(Input::Mouse(MouseInput::MouseRelease {
                    button: Ok(button),
                    coord,
                }));
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let handle_wheel = Closure::wrap(Box::new(move |event: JsValue| {
        let context = context.borrow_mut();
        let mut app = app.borrow_mut();
        let element_display_info = context.element_display_info();
        let wheel_event = event.unchecked_ref::<WheelEvent>();
        let coord = element_display_info.mouse_coord(wheel_event.client_x(), wheel_event.client_y());
        if wheel_event.delta_x() < 0. {
            app.on_input(Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Left,
                coord,
            }));
        } else if wheel_event.delta_x() > 0. {
            app.on_input(Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Right,
                coord,
            }));
        }
        if wheel_event.delta_y() < 0. {
            app.on_input(Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Up,
                coord,
            }));
        } else if wheel_event.delta_y() > 0. {
            app.on_input(Input::Mouse(MouseInput::MouseScroll {
                direction: ScrollDirection::Down,
                coord,
            }));
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
