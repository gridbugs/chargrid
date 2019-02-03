#!/bin/bash
set -euxo pipefail

WASM_FILE=drag_wasm.wasm

if [ "$#" -ne 1 ]; then
    echo "Usage $0 (release|debug)"
    exit 1
fi
MODE=$1

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TOP_LEVEL_DIR="$DIR/../../.."
WASM_DIR_RAW=$TOP_LEVEL_DIR/target/wasm32-unknown-unknown/$MODE
WASM_DIR=wasm_out

case $MODE in
    release)
        CARGO_ARGS="--release"
        ;;
    debug)
        CARGO_ARGS=""
        ;;
    *)
esac
mkdir -p $WASM_DIR
cargo build --target=wasm32-unknown-unknown $CARGO_ARGS
wasm-bindgen $WASM_DIR_RAW/$WASM_FILE --out-dir $WASM_DIR --out-name app
