import { JsGrid, JsRenderer } from 'prototty-wasm-render-js';
const wasm = import('../wasm_out/title_wasm');

wasm.then(async wasm => {
    let renderer = new JsRenderer(20, 20);
    let app = new wasm.WebApp(renderer.js_grid());
    app.run(renderer);
});
