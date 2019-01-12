import { JsGrid } from 'prototty_render';
const wasm = import('../wasm_out/title_wasm');

wasm.then(async wasm => {
    let config = {
        font_family: "Hack",
        font_size: "24px",
        cell_width_px: 14,
        cell_height_px: 28,
    };
    let js_grid = new JsGrid(app_node, 20, 20, config);
    let app = new wasm.WebApp(js_grid);
    app.run();
});
