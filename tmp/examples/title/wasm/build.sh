#!/bin/bash

CRATE="title_wasm"

cargo build \
    --target=wasm32-unknown-unknown --release

wasm-gc \
    target/wasm32-unknown-unknown/release/$CRATE.wasm \
    dist/$CRATE.wasm
