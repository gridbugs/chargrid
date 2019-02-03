'use strict';

import { Context, rngSeed } from 'prototty';
const wasm = import('../wasm_out/app');

document.oncontextmenu = () => false;

wasm.then(async wasm => {
    let config = {
        WasmInputBufferType: wasm.InputBuffer,
        node: app_node,
        grid_width: 20,
        grid_height: 20,
        font_family: "Hack",
        font_size: "24px",
        cell_width_px: 14,
        cell_height_px: 28,
    };
    let context = new Context(config);
    let app = new wasm.WebApp(rngSeed(), context.grid());
    context.run_animation((input_buffer, period) => app.tick(input_buffer, period));
});
