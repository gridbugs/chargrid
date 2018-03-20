#!/bin/bash

CRATE="tetris_wasm"

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TOP_LEVEL_DIR="$DIR/../../.."

pushd $DIR

if [[ "$1" == '--with-npm-install' ]]; then
    npm install
fi

cargo build --target=wasm32-unknown-unknown --release
wasm-gc $TOP_LEVEL_DIR/target/wasm32-unknown-unknown/release/$CRATE.wasm dist/$CRATE.wasm

npx webpack-cli

popd
