#!/bin/bash

set -euxo pipefail

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo clean

cargo test --all
cargo test --all --features=serialize
cargo test --manifest-path=file-storage/Cargo.toml --features=bincode,json,toml,yaml

cargo clean

find -name node_modules -exec rm -rf {} \; || true
find -name package-lock.json -delete || true
find -name dist -exec rm -rf {} \; || true

pushd $DIR/examples/tetris/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/title/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/fib/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/drag/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/pager/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/colour_picker/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/roguelike/wasm
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/colour_grid/wasm
npm install
npm run build -- --mode development
popd
