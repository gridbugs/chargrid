import React from 'react';
import ReactDOM from 'react-dom';

function make_colour(rgb24) {
    let r = rgb24 & 0xff;
    let g = (rgb24 >> 8) & 0xff;
    let b = (rgb24 >> 16) & 0xff;
    return `rgb(${r},${g},${b})`;
}

class Terminal extends React.Component {
    constructor(props) {
        super(props);
        this.bufs = props.bufs;
        this.ptrs = props.ptrs;
        this.mod = props.mod;
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
                let fg_colour = make_colour(this.bufs.fg_colour[index]);
                let bg_colour = make_colour(this.bufs.bg_colour[index]);

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

const DEFAULT_INPUT_BUF_SIZE = 1024;

function runProtottyApp(wasm_path, width, height, node, input_buf_size=DEFAULT_INPUT_BUF_SIZE, seed=undefined) {

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
        bufs.fg_colour = new Uint32Array(mod.memory.buffer, ptrs.fg_colour, size);
        bufs.bg_colour = new Uint32Array(mod.memory.buffer, ptrs.bg_colour, size);

        ptrs.input = mod.alloc_buf(input_buf_size);
        bufs.input = new Uint8ClampedArray(mod.memory.buffer, ptrs.input, input_buf_size);

        let props = {
            bufs,
            ptrs,
            mod,
            height,
            width,
            input_buf_size,
        };
        ReactDOM.render(React.createElement(Terminal, props), node);
    })
}

export default { runProtottyApp, DEFAULT_INPUT_BUF_SIZE };
