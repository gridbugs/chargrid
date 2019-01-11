pub extern crate prototty_render;
extern crate wasm_bindgen;

pub use prototty_render::{Coord, Size};
use prototty_render::{Rgb24, View, ViewCell, ViewGrid};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type JsGrid;

    #[wasm_bindgen(method)]
    fn js_set_cell(
        this: &JsGrid,
        x: i32,
        y: i32,
        depth: i32,
        character: JsValue,
        bold: JsValue,
        underline: JsValue,
        foreground: JsValue,
        background: JsValue,
    );

    #[wasm_bindgen(method)]
    fn js_clear(this: &JsGrid);

    #[wasm_bindgen(method)]
    fn js_render(this: &JsGrid);
}

fn rgb24_to_js_string(Rgb24 { red, green, blue }: Rgb24) -> String {
    format!("rgb({},{},{})", red, green, blue)
}

impl ViewGrid for JsGrid {
    fn set_cell(&mut self, coord: Coord, depth: i32, info: ViewCell) {
        let x = coord.x;
        let y = coord.y;
        let character = info
            .character
            .map(|character| JsValue::from_str(&character.to_string()))
            .unwrap_or(JsValue::NULL);
        let bold = info.bold.map(JsValue::from_bool).unwrap_or(JsValue::NULL);
        let underline = info
            .underline
            .map(JsValue::from_bool)
            .unwrap_or(JsValue::NULL);
        let foreground = info
            .foreground
            .map(|foreground| JsValue::from_str(&rgb24_to_js_string(foreground)))
            .unwrap_or(JsValue::NULL);
        let background = info
            .background
            .map(|background| JsValue::from_str(&rgb24_to_js_string(background)))
            .unwrap_or(JsValue::NULL);
        self.js_set_cell(
            x, y, depth, character, bold, underline, foreground, background,
        );
    }
}

impl JsGrid {
    pub fn render_at<V: View<T>, T>(&mut self, view: &mut V, data: &T, offset: Coord, depth: i32) {
        self.js_clear();
        view.view(data, offset, depth, self);
        self.js_render();
    }
    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) {
        self.render_at(view, data, Coord::new(0, 0), 0);
    }
}
