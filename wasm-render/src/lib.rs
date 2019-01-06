pub extern crate prototty_render;
extern crate wasm_bindgen;

use prototty_render::{Coord, Rgb24, View, ViewCellInfo, ViewGrid};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "prototty-wasm-render-js")]
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

    pub type JsRenderer;

    #[wasm_bindgen(method)]
    pub fn js_render(this: &JsRenderer, js_grid: &JsGrid);
}

fn rgb24_to_js_string(Rgb24 { red, green, blue }: Rgb24) -> String {
    format!("rgb({},{},{})", red, green, blue)
}

impl ViewGrid for JsGrid {
    fn set_cell(&mut self, coord: Coord, depth: i32, info: ViewCellInfo) {
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
    pub fn view_at<V: View<T>, T>(&mut self, view: &mut V, data: &T, offset: Coord, depth: i32) {
        view.view(data, offset, depth, self);
    }
    pub fn view<V: View<T>, T>(&mut self, view: &mut V, data: &T) {
        self.view_at(view, data, Coord::new(0, 0), 0);
    }
}
