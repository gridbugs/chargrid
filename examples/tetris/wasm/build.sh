#!/bin/bash

set -e

CRATE="wasm_app"

cargo build --target=wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/$CRATE.wasm dist/$CRATE.wasm
npx webpack
