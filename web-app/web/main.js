const dynenv = {};

const WIDTH = 40;
const HEIGHT = 40;
const SIZE = WIDTH * HEIGHT;

const BOLD_BIT = 1 << 0;
const UNDERLINE_BIT = 1 << 1;

const env = {
    get_width: () => WIDTH,
    get_height: () => HEIGHT,
    render: () => {
        dynenv.render();
    },
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

fetch("web_app.wasm").then(response =>
    response.arrayBuffer()
).then(bytes =>
    WebAssembly.instantiate(bytes, { env })
).then(results => {

    let mod = results.instance.exports;

    let bufs = {};
    let ansi_colour_table = make_ansi_colour_table();

    dynenv.set_bufs = (chars, style, fg_colour, bg_colour) => {
        bufs.chars = new Uint32Array(mod.memory.buffer, chars, SIZE);
        bufs.style = new Uint8Array(mod.memory.buffer, style, SIZE);
        bufs.fg_colour = new Uint8Array(mod.memory.buffer, fg_colour, SIZE);
        bufs.bg_colour = new Uint8Array(mod.memory.buffer, bg_colour, SIZE);
    };

    dynenv.render = () => {
        let s = "";
        let index = 0;
        for (let i = 0; i < HEIGHT; i++) {
            for (let j = 0; j < WIDTH; j++) {
                let code_point = bufs.chars[index];
                let style = bufs.style[index];
                let fg_colour = ansi_colour_table[bufs.fg_colour[index]];
                let bg_colour = ansi_colour_table[bufs.bg_colour[index]];

                let ch = String.fromCodePoint(code_point);
                if (ch == " ") {
                    ch = "&nbsp;"
                }

                let style_str = "";
                if (style & BOLD_BIT) {
                    style_str += "font-weight: bold;";
                }

                if (style & UNDERLINE_BIT) {
                    style_str += "text-decoration:underline;";
                }

                style_str += `color:${fg_colour};`;
                style_str += `background-color:${bg_colour};`;

                let html = `<span style="${style_str}">${ch}</span>`;

                s += html;
                index++;
            }
            s += "<br/>";
        }
        display.innerHTML = s;
    };

    let seed = parseInt(2**32 * Math.random());
    console.debug(mod);
    let app_ptr = mod.alloc_app(seed);

    mod.tick(app_ptr, null, 0, 1);
});
