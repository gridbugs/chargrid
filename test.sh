#!/bin/bash

set -e

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test
cargo test --features=serialize

cargo clean

pushd $DIR/examples/tetris/wasm
npm install
npx webpack
popd

pushd $DIR/examples/title/wasm
npm install
npx webpack
popd

pushd $DIR/examples/fib/wasm
npm install
npx webpack
popd
