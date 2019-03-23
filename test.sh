#!/bin/bash

set -euxo pipefail

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo clean

cargo test --all

cargo clean

find -name node_modules -exec rm -rf {} \; || true
find -name package-lock.json -delete || true
find -name dist -exec rm -rf {} \; || true
find -name wasm_out -exec rm -rf {} \; || true

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

pushd $DIR/examples/drag/wasm
npm install
./build_wasm.sh debug
npx webpack
popd

pushd $DIR/examples/pager/wasm
npm install
./build_wasm.sh debug
npx webpack
popd
