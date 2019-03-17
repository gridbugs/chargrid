pub extern crate prototty_render;
extern crate wasm_bindgen;

use prototty_render::*;
pub use prototty_render::{Coord, Size};
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

    #[wasm_bindgen(method)]
    fn js_width(this: &JsGrid) -> u32;

    #[wasm_bindgen(method)]
    fn js_height(this: &JsGrid) -> u32;
}

fn rgb24_to_js_string(Rgb24 { r, g, b }: Rgb24) -> String {
    format!("rgb({},{},{})", r, g, b)
}

impl ViewGrid for JsGrid {
    fn set_cell_absolute(&mut self, coord: Coord, depth: i32, view_cell: ViewCell) {
        let x = coord.x;
        let y = coord.y;
        let character = view_cell
            .character
            .map(|character| JsValue::from_str(&character.to_string()))
            .unwrap_or(JsValue::NULL);
        let bold = view_cell
            .bold
            .map(JsValue::from_bool)
            .unwrap_or(JsValue::NULL);
        let underline = view_cell
            .underline
            .map(JsValue::from_bool)
            .unwrap_or(JsValue::NULL);
        let foreground = view_cell
            .foreground
            .map(|foreground| JsValue::from_str(&rgb24_to_js_string(foreground)))
            .unwrap_or(JsValue::NULL);
        let background = view_cell
            .background
            .map(|background| JsValue::from_str(&rgb24_to_js_string(background)))
            .unwrap_or(JsValue::NULL);
        self.js_set_cell(
            x, y, depth, character, bold, underline, foreground, background,
        );
    }

    fn size(&self) -> Size {
        Size::new(self.js_width(), self.js_height())
    }
}

impl JsGrid {
    pub fn render_at<V: View<T>, T, R: ViewTransformRgb24>(
        &mut self,
        view: &mut V,
        data: &T,
        context: ViewContext<R>,
    ) {
        self.js_clear();
        view.view(data, context, self);
        self.js_render();
    }

    pub fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) {
        let size = self.size();
        self.render_at(view, data, ViewContext::default_with_size(size))
    }
}
