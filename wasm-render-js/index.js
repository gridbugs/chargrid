"use strict";

function lookup_default(obj, key, def) {
    let val = obj[key];
    if (val === undefined) {
        return def;
    } else {
        return val;
    }
}

function styleSheet(config) {
    let font_family = lookup_default(config, "font_family", "monospace");
    let font_size = lookup_default(config, "font_size", "16px");
    let style_sheet = document.createElement("style");
        style_sheet.innerHTML = `
            .prototty-terminal br {
                line-height: 0px;
                margin: 0px;
                padding: 0px;
            }
            .prototty-terminal span {
                margin: 0px;
                padding: 0px;
                font-family: ${font_family};
                font-size: ${font_size};
            }

        `;
    return style_sheet;
}

export function installStyleSheet(config) {
    document.head.appendChild(styleSheet(config));
}

class Cell {
    constructor(node) {
        this.node = node;
        this.clear();
    }
    clear() {
        this.character = "&nbsp";
        this.bold = false;
        this.underline = false;
        this.foreground = "rgb(255,255,255)";
        this.background = "rgb(0,0,0)";
        this.foreground_depth = 0;
        this.background_depth = 0;
        this.node.innerHTML = this.character;
        this.node.style.fontWeight = "normal";
        this.node.style.textDecoration = "none";
        this.node.style.color = this.foreground;
        this.node.style.backgroundColor = this.background;
    }
    set(depth, character, bold, underline, foreground, background) {
        if (depth >= this.background_depth) {
            if (background !== null && this.background !== background) {
                this.background_depth = depth;
                this.background = background;
                this.node.style.backgroundColor = this.background;
            }
        }
        if (depth >= this.foreground_depth) {
            if (character !== null && this.character !== character) {
                this.foreground_depth = depth;
                this.character = character;
                this.node.innerHTML = this.character;
            }
            if (bold !== null && this.bold !== bold) {
                this.foreground_depth = depth;
                this.bold = bold;
                if (this.bold) {
                    this.node.style.fontWeight = "bold";
                } else {
                    this.node.style.fontWeight = "normal";
                }
            }
            if (underline !== null && this.underline !== underline) {
                this.foreground_depth = depth;
                this.underline = underline;
                if (this.underline) {
                    this.node.style.textDecoration = "underline";
                } else {
                    this.node.style.textDecoration = "none";
                }
            }
            if (foreground !== null && this.foreground !== foreground) {
                this.foreground_depth = depth;
                this.foreground = foreground;
                this.node.style.color = this.foreground;
            }
        }
    }
}

export class JsGrid {
    constructor(node, width, height) {
        this.width = width;
        this.height = height;
        this.node = node;
        this.node.className = "prototty-terminal";
        this.cells = [];
        for (let i = 0; i < height; i++) {
            for (let j = 0; j < width; j++) {
                let node = document.createElement("span");
                let cell = new Cell(node);
                this.cells.push(cell);
                this.node.appendChild(node);
            }
            this.node.appendChild(document.createElement("br"));
        }
    }
    clear() {
        return;
        for (let cell of this.cells) {
            cell.set(0, "&nbsp", null, null, "rgb(255,255,255)", "rgb(0,0,0)");
        }
    }
    js_set_cell(x, y, depth, character, bold, underline, foreground, background) {
        if (x < 0 || y < 0 || x >= this.width || y >= this.height) {
            return;
        }
        if (character === " ") {
            character = "&nbsp";
        }
        let index = y * this.width + x;
        let cell = this.cells[index];
        cell.set(depth, character, bold, underline, foreground, background);
    }
}
