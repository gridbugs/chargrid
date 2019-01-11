'use strict';

function handleKeyDown(prototty_input, e) {
    prototty_input.push_key_press(e.keyCode, e.shiftKey);
}

export function registerKeyDownInputBuffer(input_buffer) {
    window.addEventListener("keydown", e => handleKeyDown(input_buffer, e));
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
        prototty_input.push_mouse_move(this.mouseCoordX(e), this.mouseCoordY(e));
    }

    handleMousePress(prototty_input, e) {
        prototty_input.push_mouse_press(this.mouseCoordX(e), this.mouseCoordY(e));
    }

    handleMouseRelease(prototty_input, e) {
        prototty_input.push_mouse_release(this.mouseCoordX(e), this.mouseCoordY(e));
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

