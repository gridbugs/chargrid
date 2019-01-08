"use strict";

export class ProtottyInput {
    constructor(buffer_a, buffer_b) {
        this.current = buffer_a;
        this.next = buffer_b;
    }

    swap_buffers() {
        this.next.clear();
        let tmp = this.current;
        this.current = this.next;
        this.next = tmp;
        return tmp;
    }

    register() {
        window.addEventListener("keydown", (e) => handleKeyDown(this, e));
    }
}

function handleKeyDown(prototty_input, e) {
    let key_code = e.keyCode;
    let shift = e.shiftKey;
    prototty_input.current.push_key_press(key_code, shift);
}
