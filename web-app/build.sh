#!/bin/bash

set -e

CRATE="web_app"

cargo build --target=wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/$CRATE.wasm web/$CRATE.wasm
