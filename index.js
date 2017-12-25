import React from 'react';
import ReactDOM from 'react-dom';

class Terminal extends React.Component {
    constructor(props) {
        super(props);
        this.bufs = props.bufs;
        this.ptrs = props.ptrs;
        this.mod = props.mod;
        this.colour_table = props.colour_table;
        this.height = props.height;
        this.width = props.width;
        this.children = new Array((this.width + 1) * this.height);
        this.num_inputs = 0;
        this.input_buf_size = props.input_buf_size;
        this.class_table = [
            "",
            "bold",
            "underline",
            "bold underline",
        ];

        let style_sheet = document.createElement("style");
        style_sheet.type = "text/css";
        style_sheet.innerHTML = `
            .prototty-terminal {
                font-family: monospace;
                font-size: 16px;
            }
            .prototty-terminal .bold {
                font-weight:bold;
            }
            .prototty-terminal .underline {
                text-decoration:underline;
            }
        `;
        this.style_sheet = style_sheet;
        this.root_props = { className: "prototty-terminal" };

        this.state = { previousInstant: Date.now() };
    }

    componentDidMount() {
        this.animationRequest = requestAnimationFrame(() => this.tick());
        window.addEventListener("keydown", (e) => this.handleKeyDown(e));
        document.head.appendChild(this.style_sheet);
    }

    componentWillUnmount() {
        cancelAnimationFrame(this.animationRequest);
        window.removeEventListener("keydown");
        document.head.removeChild(this.style_sheet);
    }

    handleKeyDown(e) {
        if (this.num_inputs < this.input_buf_size) {
            this.bufs.input[this.num_inputs] = e.keyCode;
            this.num_inputs += 1;
        }
    }

    tick() {
        let now = Date.now();
        let period = now - this.state.previousInstant;

        this.mod.tick(this.ptrs.app, this.ptrs.input, this.num_inputs, period);
        this.num_inputs = 0;

        this.setState(_ => ({ previousInstant: now }));
        this.animationRequest = requestAnimationFrame(() => this.tick());
    }

    render() {
        let index = 0;
        let out_index = 0;
        for (let i = 0; i < this.height; i++) {
            for (let j = 0; j < this.width; j++) {

                let code_point = this.bufs.chars[index];
                let style = this.bufs.style[index];
                let fg_colour = this.colour_table[this.bufs.fg_colour[index]];
                let bg_colour = this.colour_table[this.bufs.bg_colour[index]];

                let ch = String.fromCodePoint(code_point);
                if (ch == " ") {
                    ch = "\u00a0";
                }

                this.children[out_index] = React.createElement(
                    "span",
                    {
                        className: this.class_table[style],
                        style: {
                            color: fg_colour,
                            backgroundColor: bg_colour,
                        },
                        key: out_index,
                    },
                    ch,
                );

                index += 1;
                out_index += 1;
            }
            this.children[out_index] = React.createElement("br", { key: out_index });
            out_index += 1;
        }

        return React.createElement("div", this.root_props, this.children);
    }
}

function make_ansi_colour_table() {
    let normal = [
        [0, 0, 0],          // Black
        [187, 0, 0],        // Red
        [0, 187, 0],        // Green
        [187, 187, 0],      // Yellow
        [0, 0, 187],        // Blue
        [187, 0, 187],      // Magenta
        [0, 187, 187],      // Cyan
        [187, 187, 187],    // Grey
    ];
    let bright = [
        [85, 85, 85],       // Dark Grey
        [255, 85, 85],      // Bright Red
        [0, 255, 0],        // Bright Green
        [255, 255, 85],     // Bright Yellow
        [85, 85, 255],      // Bright Blue
        [255, 85, 255],     // Bright Magenta
        [85, 255, 255],     // Bright Cyan
        [255, 255, 255],    // White
    ];

    let rgb_steps = [0, 51, 102, 153, 204, 255];
    let rgb = [];
    for (let r of rgb_steps) {
        for (let g of rgb_steps) {
            for (let b of rgb_steps) {
                rgb.push([r, g, b]);
            }
        }
    }

    let num_grey_scale = 24;
    let grey_scale = [];
    for (let i = 0; i < num_grey_scale; i++) {
        let x = i * (255 / num_grey_scale);
        grey_scale.push([x, x, x]);
    }

    let all = normal.concat(bright).concat(rgb).concat(grey_scale);
    let all_strings = all.map(([r, g, b]) => `rgb(${r},${g},${b})`);

    return all_strings;
}

const DEFAULT_COLOUR_TABLE = make_ansi_colour_table();
const DEFAULT_INPUT_BUF_SIZE = 1024;

function runProtottyApp(wasm_path, width, height, node, colour_table=DEFAULT_COLOUR_TABLE, input_buf_size=DEFAULT_INPUT_BUF_SIZE, seed=undefined) {

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
    };

    return fetch(wasm_path).then(response =>
        response.arrayBuffer()
    ).then(bytes =>
        WebAssembly.instantiate(bytes, { env })
    ).then(results => {

        let mod = results.instance.exports;

        ptrs.app = mod.alloc_app(seed);

        bufs.chars = new Uint32Array(mod.memory.buffer, ptrs.chars, size);
        bufs.style = new Uint8Array(mod.memory.buffer, ptrs.style, size);
        bufs.fg_colour = new Uint8Array(mod.memory.buffer, ptrs.fg_colour, size);
        bufs.bg_colour = new Uint8Array(mod.memory.buffer, ptrs.bg_colour, size);

        ptrs.input = mod.alloc_buf(input_buf_size);
        bufs.input = new Uint8ClampedArray(mod.memory.buffer, ptrs.input, input_buf_size);

        let props = {
            bufs,
            ptrs,
            mod,
            colour_table,
            height,
            width,
            input_buf_size,
        };
        ReactDOM.render(React.createElement(Terminal, props), node);
    })
}

export default { runProtottyApp, DEFAULT_COLOUR_TABLE, DEFAULT_INPUT_BUF_SIZE };
