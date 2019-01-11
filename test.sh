#!/bin/bash

set -euxo pipefail

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test

cargo clean

pushd $DIR/examples/tetris/wasm
npm install
./build_wasm.sh debug
npx webpack
popd

pushd $DIR/examples/title/wasm
npm install
./build_wasm.sh debug
npx webpack
popd

pushd $DIR/examples/fib/wasm
npm install
./build_wasm.sh debug
npx webpack
popd
