#!/bin/bash

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

pushd $DIR

TOP_LEVEL_DIR="$DIR/../../.."

CRATE="tetris_wasm"

if [[ "$1" == '--with-npm-install' ]]; then
    npm install
fi

cargo build --target=wasm32-unknown-unknown --release
wasm-gc $TOP_LEVEL_DIR/target/wasm32-unknown-unknown/release/$CRATE.wasm dist/$CRATE.wasm

npx webpack

popd
