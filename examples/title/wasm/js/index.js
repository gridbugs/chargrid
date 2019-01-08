import { JsGrid, JsRenderer, installStyleSheet } from 'prototty-wasm-render-js';
const wasm = import('../wasm_out/title_wasm');

wasm.then(async wasm => {
    installStyleSheet({font_family: "Hack", font_size: "24px"});
    let js_grid = new JsGrid(app_node, 20, 20);
    let app = new wasm.WebApp(js_grid);
    app.run();
});
