"use strict";

const MOD_SHIFT = (1 << 0);

function make_colour(rgb24) {
    let r = rgb24 & 0xff;
    let g = (rgb24 >> 8) & 0xff;
    let b = (rgb24 >> 16) & 0xff;
    return `rgb(${r},${g},${b})`;
}

class Terminal {
    constructor(props) {
        this.node = props.node;
        this.bufs = props.bufs;
        this.ptrs = props.ptrs;
        this.mod = props.mod;
        this.height = props.height;
        this.width = props.width;
        this.size = this.width * this.height;
        this.num_inputs = 0;
        this.input_buf_size = props.input_buf_size;
        this.class_table = [
            "cell",
            "cell bold",
            "cell underline",
            "cell bold underline",
        ];

        let style_sheet = document.createElement("style");
        style_sheet.type = "text/css";

        let make_style = obj => Object.keys(obj).map(k => `${k}:${obj[k]};`).join("");

        style_sheet.innerHTML = `
            .prototty-terminal {
                ${make_style(props.style)}
            }
            .prototty-terminal .cell {
                ${make_style(props.cell_style)}
            }
            .prototty-terminal .bold {
                ${make_style(props.bold_style)}
            }
            .prototty-terminal .underline {
                ${make_style(props.underline_style)}
            }
            .prototty-terminal br {
                line-height: 0px;
                margin: 0px;
                padding: 0px;
            }
        `;
        this.style_sheet = style_sheet;
        this.node.className = "prototty-terminal";

        this.children = new Array(this.size);
        let index = 0;
        for (let i = 0; i < this.height; i++) {
            for (let j = 0; j < this.width; j++) {
                let child = document.createElement("span");
                child.className = "cell";
                this.node.appendChild(child);
                this.children[index] = {
                    node: child,
                    code_point: 0,
                    style: 0,
                    fg_colour: 0,
                    bg_colour: 0,
                };
                index += 1;
            }
            this.node.appendChild(document.createElement("br"));
        }
    }

    start() {
        this.animationRequest = requestAnimationFrame(() => this.tick());
        window.addEventListener("keydown", (e) => this.handleKeyDown(e));
        document.head.appendChild(this.style_sheet);
        this.previous_instant = Date.now();
    }

    stop() {
        cancelAnimationFrame(this.animationRequest);
        window.removeEventListener("keydown");
        document.head.removeChild(this.style_sheet);
    }

    handleKeyDown(e) {
        if (this.num_inputs < this.input_buf_size) {
            this.bufs.key_code_buffer[this.num_inputs] = e.keyCode;

            let key_mod = 0;

            if (e.shiftKey) {
                key_mod |= MOD_SHIFT;
            }

            this.bufs.key_mod_buffer[this.num_inputs] = key_mod;

            this.num_inputs += 1;
        }
    }

    tick() {
        let now = Date.now();
        let period = now - this.previous_instant;

        this.mod.tick(this.ptrs.app, this.ptrs.key_code_buffer, this.ptrs.key_mod_buffer, this.num_inputs, period);
        this.num_inputs = 0;

        this.previous_instant = now;

        this.render();

        this.animationRequest = requestAnimationFrame(() => this.tick());
    }

    render() {
        let index = 0;
        for (let i = 0; i < this.height; i++) {
            for (let j = 0; j < this.width; j++) {

                let code_point = this.bufs.chars[index];
                let style = this.bufs.style[index];
                let fg_colour_code = this.bufs.fg_colour[index];
                let bg_colour_code = this.bufs.bg_colour[index];

                let child = this.children[index];

                if (child.code_point != code_point) {
                    child.code_point = code_point;

                    let ch = String.fromCodePoint(code_point);
                    if (ch == " ") {
                        ch = "\u00a0";
                    }
                    child.node.innerHTML = ch;
                }

                if (child.style != style) {
                    child.style = style;
                    child.node.className = this.class_table[style];
                }

                if (child.fg_colour != fg_colour_code) {
                    child.fg_colour = fg_colour_code;
                    let fg_colour = make_colour(fg_colour_code);
                    child.node.style.color = fg_colour;
                }

                if (child.bg_colour != bg_colour_code) {
                    child.bg_colour = bg_colour_code;
                    let bg_colour = make_colour(bg_colour_code);
                    child.node.style.backgroundColor = bg_colour;
                }

                index += 1;
            }
        }
    }
}

const DEFAULT_INPUT_BUF_SIZE = 1024;

function loolkup_default(obj, key, def) {
    let val = obj[key];
    if (val === undefined) {
        return def;
    } else {
        return val;
    }
}

function init_env_fn(config, name) {
    return loolkup_default(config, name, () => {});
}

function loadProtottyApp(wasm_path, width, height, node, config={}, input_buf_size=DEFAULT_INPUT_BUF_SIZE, seed=undefined) {

    const size = width * height;

    if (seed == undefined) {
        seed = parseInt(2**32 * Math.random());
    }

    const bufs = {};
    const ptrs = {};

    const env = {
        get_width: () => width,
        get_height: () => width,
        set_bufs: (chars, style, fg_colour, bg_colour) => {
            ptrs.chars = chars;
            ptrs.style = style;
            ptrs.fg_colour = fg_colour;
            ptrs.bg_colour = bg_colour;
        },
        quit: init_env_fn(config, "quit"),
    };

    let style = loolkup_default(config, "style", { "line-height": "1em" });
    let cell_style = loolkup_default(config, "cell_style", { "font-family": "monospace", "font-size": "16px" });
    let bold_style = loolkup_default(config, "bold_style", { "font-weight": "bold" });
    let underline_style = loolkup_default(config, "underline_style", { "text-decoration": "underline" });

    return fetch(wasm_path).then(response =>
        response.arrayBuffer()
    ).then(bytes =>
        WebAssembly.instantiate(bytes, { env })
    ).then(results => {

        let mod = results.instance.exports;

        ptrs.app = mod.alloc_app(seed);

        bufs.chars = new Uint32Array(mod.memory.buffer, ptrs.chars, size);
        bufs.style = new Uint8Array(mod.memory.buffer, ptrs.style, size);
        bufs.fg_colour = new Uint32Array(mod.memory.buffer, ptrs.fg_colour, size);
        bufs.bg_colour = new Uint32Array(mod.memory.buffer, ptrs.bg_colour, size);

        ptrs.key_mod_buffer = mod.alloc_byte_buffer(input_buf_size);
        bufs.key_mod_buffer = new Uint8ClampedArray(mod.memory.buffer, ptrs.key_mod_buffer, input_buf_size);

        ptrs.key_code_buffer = mod.alloc_byte_buffer(input_buf_size);
        bufs.key_code_buffer = new Uint8ClampedArray(mod.memory.buffer, ptrs.key_code_buffer, input_buf_size);

        let props = {
            bufs,
            ptrs,
            mod,
            height,
            width,
            input_buf_size,
            node,
            style,
            cell_style,
            bold_style,
            underline_style,
        };

        let terminal = new Terminal(props);

        return {
            start: () => terminal.start(),
            stop: () => terminal.stop(),
        };
    })
}

export default { loadProtottyApp, DEFAULT_INPUT_BUF_SIZE };
