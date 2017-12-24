import React from 'react';
import ReactDOM from 'react-dom';

const INPUT_BUF_SIZE = 1024;

class Terminal extends React.Component {
    constructor(props) {
        super(props);
        this.bufs = props.bufs;
        this.ptrs = props.ptrs;
        this.mod = props.mod;
        this.colour_table = props.colour_table;
        this.class_table = props.class_table;
        this.height = props.height;
        this.width = props.width;
        this.children = new Array((this.width + 1) * this.height);
        this.num_inputs = 0;
        this.state = { previousInstant: Date.now() };
    }

    componentDidMount() {
        this.animationRequest = requestAnimationFrame(() => this.tick());
        window.addEventListener("keydown", (e) => this.handleKeyDown(e));
    }

    componentWillUnmount() {
        cancelAnimationFrame(this.animationRequest);
        window.removeEventListener("keydown");
    }

    handleKeyDown(e) {
        if (this.num_inputs < INPUT_BUF_SIZE) {
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

        return React.createElement("div", null, this.children);
    }
}

const WIDTH = 20;
const HEIGHT = 20;
const SIZE = WIDTH * HEIGHT;

const BOLD_BIT = 1 << 0;
const UNDERLINE_BIT = 1 << 1;

const dynenv = {};
const env = {
    get_width: () => WIDTH,
    get_height: () => HEIGHT,
    set_bufs: (chars, style, fg_colour, bg_colour) => {
        dynenv.set_bufs(chars, style, fg_colour, bg_colour);
    },
};

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

fetch("wasm_app.wasm").then(response =>
    response.arrayBuffer()
).then(bytes =>
    WebAssembly.instantiate(bytes, { env })
).then(results => {

    let mod = results.instance.exports;

    let bufs = {};
    let ptrs = {};
    let colour_table = make_ansi_colour_table();

    dynenv.set_bufs = (chars, style, fg_colour, bg_colour) => {
        bufs.chars = new Uint32Array(mod.memory.buffer, chars, SIZE);
        bufs.style = new Uint8Array(mod.memory.buffer, style, SIZE);
        bufs.fg_colour = new Uint8Array(mod.memory.buffer, fg_colour, SIZE);
        bufs.bg_colour = new Uint8Array(mod.memory.buffer, bg_colour, SIZE);
    };

    let seed = parseInt(2**32 * Math.random());
    ptrs.app = mod.alloc_app(seed);
    ptrs.input = mod.alloc_buf(INPUT_BUF_SIZE);
    bufs.input = new Uint8ClampedArray(mod.memory.buffer, ptrs.input, INPUT_BUF_SIZE);

    let class_table = [
        "",
        "bold",
        "underline",
        "bold underline",
    ];

    let props = {
        bufs,
        ptrs,
        mod,
        colour_table,
        class_table,
        height: HEIGHT,
        width: WIDTH,
    };
    ReactDOM.render(React.createElement(Terminal, props), mountNode);
});
