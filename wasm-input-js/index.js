'use strict';

function handleKeyDown(prototty_input, e) {
    prototty_input.push_key_press(e.keyCode, e.shiftKey);
}

export function registerKeyDownInputBuffer(input_buffer) {
    window.addEventListener("keydown", e => handleKeyDown(input_buffer, e));
}

const LEFT_BIT = 1;
const RIGHT_BIT = 2;
const MIDDLE_BIT = 4;
const MASK = (1 << LEFT_BIT) | (1 << RIGHT_BIT) | (1 << MIDDLE_BIT);

const LEFT = 0;
const RIGHT = 2;
const MIDDLE = 1;

function contains_left(e) {
    return (e.buttons & (1 << LEFT_BIT)) !== 0;
}
function contains_right(e) {
    return (e.buttons & (1 << RIGHT_BIT)) !== 0;
}
function contains_middle(e) {
    return (e.buttons & (1 << MIDDLE_BIT)) !== 0;
}
function contains_none(e) {
    return (e.buttons & MASK) === 0;
}

export class InputContext {
    constructor(grid_x, grid_y, cell_width, cell_height) {
        this.grid_x = grid_x;
        this.grid_y = grid_y;
        this.cell_width = cell_width;
        this.cell_height = cell_height;
    }

    mouseCoordX(e) {
        return parseInt((e.clientX - this.grid_x) / this.cell_width);
    }

    mouseCoordY(e) {
        return parseInt((e.clientY - this.grid_y) / this.cell_height);
    }

    handleMouseMove(prototty_input, e) {
        if (contains_left(e)) {
            prototty_input.push_mouse_move_left(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (contains_right(e)) {
            prototty_input.push_mouse_move_right(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (contains_middle(e)) {
            prototty_input.push_mouse_move_middle(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (contains_none(e)) {
            prototty_input.push_mouse_move_none(this.mouseCoordX(e), this.mouseCoordY(e));
        }
    }

    handleMousePress(prototty_input, e) {
        if (e.button === LEFT) {
            prototty_input.push_mouse_press_left(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (e.button === RIGHT) {
            prototty_input.push_mouse_press_right(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (e.button === MIDDLE) {
            prototty_input.push_mouse_press_middle(this.mouseCoordX(e), this.mouseCoordY(e));
        }
    }

    handleMouseRelease(prototty_input, e) {
        if (e.button === LEFT) {
            prototty_input.push_mouse_release_left(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (e.button === RIGHT) {
            prototty_input.push_mouse_release_right(this.mouseCoordX(e), this.mouseCoordY(e));
        }
        if (e.button === MIDDLE) {
            prototty_input.push_mouse_release_middle(this.mouseCoordX(e), this.mouseCoordY(e));
        }
    }

    handleMouseWheel(prototty_input, e) {
        let x = this.mouseCoordX(e);
        let y = this.mouseCoordY(e);
        if (e.deltaX < 0) {
            prototty_input.push_mouse_scroll_left(x, y);
        } else if (e.deltaX > 0) {
            prototty_input.push_mouse_scroll_right(x, y);
        }
        if (e.deltaY < 0) {
            prototty_input.push_mouse_scroll_up(x, y);
        } else if (e.deltaY > 0) {
            prototty_input.push_mouse_scroll_down(x, y);
        }
    }

    setInputBuffer(input_buffer) {
        registerKeyDownInputBuffer(input_buffer);
        window.addEventListener("mousemove", e => this.handleMouseMove(input_buffer, e));
        window.addEventListener("mousedown", e => this.handleMousePress(input_buffer, e));
        window.addEventListener("mouseup", e => this.handleMouseRelease(input_buffer, e));
        window.addEventListener("mousewheel", e => this.handleMouseWheel(input_buffer, e));
    }
}

