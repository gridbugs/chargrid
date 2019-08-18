extern crate grid_2d;
extern crate js_sys;
pub extern crate prototty_event_routine;
extern crate prototty_grid;
pub extern crate prototty_input;
pub extern crate prototty_render;
extern crate prototty_storage;
extern crate wasm_bindgen;
extern crate web_sys;

mod input;

use grid_2d::Coord;
pub use grid_2d::Size;
use js_sys::Function;
use prototty_event_routine::{common_event::CommonEvent, Event, EventRoutine, Handled};
pub use prototty_input::{Input, MouseInput};
use prototty_input::{MouseButton, ScrollDirection};
use prototty_render::{ColModify, Frame, Rgb24, View, ViewCell, ViewContext, ViewContextDefault};
use prototty_storage::*;
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
    prototty_grid: prototty_grid::Grid,
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
        let prototty_grid = prototty_grid::Grid::new(size);
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

    pub fn render_at<V: View<T>, T, C: ColModify>(&mut self, view: &mut V, data: T, context: ViewContext<C>) {
        self.prototty_grid.clear();
        view.view(data, context, &mut self.prototty_grid);
        self.render_internal();
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: T) {
        let context = self.default_context();
        self.render_at(view, data, context);
    }

    pub fn frame(&mut self) -> WasmFrame {
        self.prototty_grid.clear();
        WasmFrame { context: self }
    }

    pub fn run_event_routine_one_shot_ignore_return<E>(self, event_routine: E, data: E::Data, view: E::View)
    where
        E: EventRoutine<Event = CommonEvent> + 'static,
    {
        let wasm_event_routine = WasmEventRoutineOneShotIgnoreReturn {
            event_routine: Some(event_routine),
            data,
            view,
        };
        self.run_event_handler(wasm_event_routine);
    }
    pub fn run_event_routine_repeating<E, F>(self, event_routine: E, data: E::Data, view: E::View, f: F)
    where
        E: EventRoutine<Event = CommonEvent> + 'static,
        F: FnMut(E::Return) -> E + 'static,
    {
        let wasm_event_routine = WasmEventRoutineRepeating {
            event_routine: Some(event_routine),
            data,
            view,
            f,
        };
        self.run_event_handler(wasm_event_routine);
    }
    pub fn run_event_handler<E>(self, event_handler: E)
    where
        E: EventHandler + 'static,
    {
        let event_handler = Rc::new(RefCell::new(event_handler));
        let context = Rc::new(RefCell::new(self));
        run_frame_handler(event_handler.clone(), context.clone());
        run_input_handler(event_handler, context);
    }
}

struct WasmEventRoutineOneShotIgnoreReturn<E>
where
    E: EventRoutine<Event = CommonEvent>,
{
    event_routine: Option<E>,
    data: E::Data,
    view: E::View,
}

impl<E> EventHandler for WasmEventRoutineOneShotIgnoreReturn<E>
where
    E: EventRoutine<Event = CommonEvent>,
{
    fn on_input(&mut self, input: Input, _context: &mut Context) {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            match event_routine.handle(&mut self.data, &mut self.view, Event::new(input.into())) {
                Handled::Continue(event_routine) => Some(event_routine),
                Handled::Return(_) => None,
            }
        } else {
            None
        };
    }
    fn on_frame(&mut self, since_last_frame: Duration, context: &mut Context) {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            match event_routine.handle(&mut self.data, &mut self.view, Event::new(since_last_frame.into())) {
                Handled::Continue(event_routine) => {
                    let mut frame = context.frame();
                    event_routine.view(&self.data, &mut self.view, frame.default_context(), &mut frame);
                    frame.render();
                    Some(event_routine)
                }
                Handled::Return(_) => None,
            }
        } else {
            None
        };
    }
}

struct WasmEventRoutineRepeating<E, F>
where
    E: EventRoutine<Event = CommonEvent>,
    F: FnMut(E::Return) -> E,
{
    event_routine: Option<E>,
    data: E::Data,
    view: E::View,
    f: F,
}

impl<E, F> EventHandler for WasmEventRoutineRepeating<E, F>
where
    E: EventRoutine<Event = CommonEvent>,
    F: FnMut(E::Return) -> E,
{
    fn on_input(&mut self, input: Input, _context: &mut Context) {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            match event_routine.handle(&mut self.data, &mut self.view, Event::new(input.into())) {
                Handled::Continue(event_routine) => Some(event_routine),
                Handled::Return(r) => Some((self.f)(r)),
            }
        } else {
            None
        };
    }
    fn on_frame(&mut self, since_last_frame: Duration, context: &mut Context) {
        self.event_routine = if let Some(event_routine) = self.event_routine.take() {
            let event_routine =
                match event_routine.handle(&mut self.data, &mut self.view, Event::new(since_last_frame.into())) {
                    Handled::Continue(event_routine) => event_routine,
                    Handled::Return(r) => (self.f)(r),
                };
            let mut frame = context.frame();
            event_routine.view(&self.data, &mut self.view, frame.default_context(), &mut frame);
            frame.render();
            Some(event_routine)
        } else {
            None
        };
    }
}

pub struct WasmFrame<'a> {
    context: &'a mut Context,
}

impl<'a> WasmFrame<'a> {
    pub fn render(self) {
        self.context.render_internal();
    }
    pub fn size(&self) -> Size {
        self.context.prototty_grid.size()
    }
    pub fn default_context(&self) -> ViewContextDefault {
        ViewContext::default_with_size(self.size())
    }
}

impl<'a> Frame for WasmFrame<'a> {
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i32, absolute_cell: ViewCell) {
        self.context
            .prototty_grid
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
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
                event_handler.on_input(
                    Input::Mouse(MouseInput::MouseMove { button: None, coord }),
                    &mut *context,
                );
            }
            if buttons::has_left(buttons) {
                event_handler.on_input(
                    Input::Mouse(MouseInput::MouseMove {
                        button: Some(MouseButton::Left),
                        coord,
                    }),
                    &mut *context,
                );
            }
            if buttons::has_right(buttons) {
                event_handler.on_input(
                    Input::Mouse(MouseInput::MouseMove {
                        button: Some(MouseButton::Right),
                        coord,
                    }),
                    &mut *context,
                );
            }
            if buttons::has_middle(buttons) {
                event_handler.on_input(
                    Input::Mouse(MouseInput::MouseMove {
                        button: Some(MouseButton::Middle),
                        coord,
                    }),
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
                event_handler.on_input(Input::Mouse(MouseInput::MousePress { button, coord }), &mut *context);
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
                    Input::Mouse(MouseInput::MouseRelease {
                        button: Ok(button),
                        coord,
                    }),
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
                Input::Mouse(MouseInput::MouseScroll {
                    direction: ScrollDirection::Left,
                    coord,
                }),
                &mut *context,
            );
        } else if wheel_event.delta_x() > 0. {
            event_handler.on_input(
                Input::Mouse(MouseInput::MouseScroll {
                    direction: ScrollDirection::Right,
                    coord,
                }),
                &mut *context,
            );
        }
        if wheel_event.delta_y() < 0. {
            event_handler.on_input(
                Input::Mouse(MouseInput::MouseScroll {
                    direction: ScrollDirection::Up,
                    coord,
                }),
                &mut *context,
            );
        } else if wheel_event.delta_y() > 0. {
            event_handler.on_input(
                Input::Mouse(MouseInput::MouseScroll {
                    direction: ScrollDirection::Down,
                    coord,
                }),
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

pub struct LocalStorage {
    local_storage: web_sys::Storage,
}

impl LocalStorage {
    pub fn new() -> Self {
        Self {
            local_storage: web_sys::window().unwrap().local_storage().unwrap().unwrap(),
        }
    }
}

impl Storage for LocalStorage {
    fn exists<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        self.local_storage.get_item(key.as_ref()).unwrap().is_some()
    }

    fn clear(&mut self) {
        self.local_storage.clear().unwrap()
    }

    fn remove<K>(&mut self, key: K) -> Result<(), RemoveError>
    where
        K: AsRef<str>,
    {
        self.local_storage
            .remove_item(key.as_ref())
            .map_err(|_| RemoveError::IoError)
    }

    fn load_raw<K>(&self, key: K) -> Result<Vec<u8>, LoadRawError>
    where
        K: AsRef<str>,
    {
        let maybe_string = self
            .local_storage
            .get_item(key.as_ref())
            .map_err(|_| LoadRawError::IoError)?;
        let string = maybe_string.ok_or(LoadRawError::NoSuchKey)?;
        serde_json::from_str(&string).map_err(|_| LoadRawError::IoError)
    }

    fn store_raw<K, V>(&mut self, key: K, value: V) -> Result<(), StoreRawError>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let string = serde_json::to_string(value.as_ref()).map_err(|_| StoreRawError::IoError)?;
        self.local_storage
            .set_item(key.as_ref(), &string)
            .map_err(|_| StoreRawError::IoError)
    }
}
