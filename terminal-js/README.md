# Prototty JS App Harness

[![Download](https://img.shields.io/npm/v/prototty-terminal-js.svg)](https://www.npmjs.com/package/prototty-terminal-js)

[Prototty](https://github.com/stevebob/prototty/) is a rust library for
rendering to a terminal, designed specifically with game prototyping in mind.
It supports compiling to web assembly. This library
provides an application harness, that takes a compiled web assembly file, and
runs it, passing through keyboard input, and rendering the output to the screen.

## Example

This runs an app in a file called "wasm\_app.wasm", rendering the output into an
element called "protottyTerminal", using a grid of 30x20 cells.

```js
import prototty from 'prototty-terminal-js';

let node = document.getElementById("protottyTerminal");
prototty.loadProtottyApp("wasm_app.wasm", 30, 20, node).then(app => app.start());
```

## Big Example

https://github.com/stevebob/protrotty/examples/tetris

This is an implementation of tetris. There's a unix app that runs in an ansi terminal,
and a wasm app which runs in a browser using this library.

## Interface

The app harness expects the following functions to be exposed by the wasm blob:

```rust
#[no_mangle]
pub extern "C" fn alloc_app(rng_seed: usize, storage_buf: *const u8, storage_len: usize) -> *mut c_void {
    // Called once during initialisation.
    // `seed` is a random integer created using Math.random(), which can be used
    // as a seed for an rng.
    // `storage_buf` and `storage_len` is an address and size suitable for passing to
    // `prototty_wasm::WasmStorage::from_ptr`.
    // Allocate your application's state, and return a raw pointer to it.
}

#[no_mangle]
pub fn tick(app: *mut c_void,
            key_code_buffer: *const u8,
            key_mod_buffer: *const u8,
            num_inputs: usize,
            period_millis: f64) {

    // Called by the app harness once per frame.
    // `app` is a pointer to your application's state, returned by `alloc_app`.
    // `key_code_buffer` is a buffer containing `event.keyCode` values of keypress events since the last frame.
    // `key_mod_buffer` is a buffer of bytes describing modifier keys:
    //    bit 0 is set <=> shift is pressed
    // `num_inputs` is the number of keycodes in `which_buffer` and `key_code_buffer`.
    // `period_millis` is the number of milliseconds that have passed since the last call to `tick`.
}
```
