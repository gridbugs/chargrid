'use strict';

export * from 'prototty_storage';
export * from 'prototty_render';
export * from 'prototty_input';

import { JsGrid, DEFAULT_CONFIG as RENDER_DEFAULT_CONFIG } from 'prototty_render';
import { JsByteStorage } from 'prototty_storage';
import { InputContext } from 'prototty_input';

export const DEFAULT_CONFIG = {
    WasmInputBufferType: null,
    node: null,
    grid_width: null,
    grid_height: null,
    font_family: RENDER_DEFAULT_CONFIG.font_family,
    font_size: RENDER_DEFAULT_CONFIG.font_size,
    cell_width_px: RENDER_DEFAULT_CONFIG.cell_width_px,
    cell_height_px: RENDER_DEFAULT_CONFIG.cell_height_px,
}

export function rngSeed() {
    return parseInt((2 ** 32) * Math.random());
}

export class Context {
    constructor(config) {
        for (let key in DEFAULT_CONFIG) {
            if (config[key] === undefined) {
                config[key] = DEFAULT_CONFIG[key];
            }
        }
        if (config.WasmInputBufferType === null) {
            throw "WasmInputBufferType config field must be populated";
        }
        if (config.node === null) {
            throw "node config field must be populated";
        }
        if (config.grid_width === null) {
            throw "grid_width config field must be populated";
        }
        if (config.grid_height === null) {
            throw "grid_height config field must be populated";
        }
        this.js_grid = new JsGrid(config.node, config.grid_width, config.grid_height, config);
        this.input_context = new InputContext(
            this.js_grid.nodeXOffset(),
            this.js_grid.nodeYOffset(),
            this.js_grid.nodeCellWidth(),
            this.js_grid.nodeCellHeight(),
        );
        this.input_buffer = new config.WasmInputBufferType();
        this.input_context.setInputBuffer(this.input_buffer);
        this.js_byte_storage = null;
    }

    async with_storage(storage_key) {
        this.js_byte_storage = await JsByteStorage.make_async(storage_key);
        return this;
    }

    grid() {
        return this.js_grid;
    }

    storage() {
        if (this.js_byte_storage === null) {
            throw "Context.with_storage must by called asynchronously before this method can be called.";
        }
        return this.js_byte_storage;
    }

    run_animation(on_tick) {
        let previous_instant = Date.now();
        let tick = () => {
            let current_instant = Date.now();
            on_tick(this.input_buffer, current_instant - previous_instant);
            this.input_buffer.clear();
            previous_instant = current_instant;
            requestAnimationFrame(tick);
        };
        tick();
    }
}
