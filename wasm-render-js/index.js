"use strict";

export class JsGrid {
    constructor(width, height) {}
    js_set_cell(x, y, depth, character, bold, underline, foreground, background) {
        console.log(x, y, depth, character, bold, underline, foreground, background);
    }
}

export class JsRenderer {
    constructor(width, height) {
        this.width = width;
        this.height = height;
    }
    js_grid() {
        return new JsGrid(this.width, this.height);
    }
    js_render(js_grid) {
        console.log(js_grid);
    }
}
