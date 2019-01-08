import { JsGrid, JsRenderer, installStyleSheet } from 'prototty-wasm-render-js';
import { ProtottyInput } from 'prototty-wasm-input-js';
const wasm = import('../wasm_out/tetris_wasm');

wasm.then(async wasm => {
    installStyleSheet({font_family: "Hack", font_size: "24px"});
    let input = new ProtottyInput(new wasm.InputBuffer(), new wasm.InputBuffer());
    input.register();
    let js_grid = new JsGrid(app_node, 20, 20);
    let seed = parseInt(2**32 * Math.random());
    let app = new wasm.WebApp(seed, js_grid);
    let previous_instant = Date.now();
    let tick = () => {
        let current_instant = Date.now();
        app.tick(input.swap_buffers(), current_instant - previous_instant);
        previous_instant = current_instant;
        requestAnimationFrame(tick);
    };
    tick()
});
