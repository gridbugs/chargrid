# prototty\_wasm

[![Version](https://img.shields.io/crates/v/prototty_wasm.svg)](https://crates.io/crates/prototty_wasm)
[![Documentation](https://docs.rs/prototty_wasm/badge.svg)](https://docs.rs/prototty_wasm)

A prototty frontend for web assembly. It renders views into memory in a format
which can be easily unpacked in javascript, and contains functions for
normalizing javascript key codes and key mods into prototty's input type. It's
intended to be used with [prototty-terminal-js](https://github.com/stevebob/prototty-terminal-js) - a
javascript library for unpacking the output of prototty-wasm and drawing it to
the screen, and periodically sending input to the wasm program.

## Example

Let's continue the title example started
[here](https://github.com/stevebob/prototty/tree/master/prototty#example):

TODO
