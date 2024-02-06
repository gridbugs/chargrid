#!/usr/bin/env bash

set -euxo pipefail

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo clean
cargo test --workspace

cargo clean
cargo test --workspace --features serialize

cargo clean

find . -name node_modules -exec rm -rf {} \; || true
find . -name dist -exec rm -rf {} \; || true

pushd $DIR/examples/component_experiment/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/colour_grid/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/tetris/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/text_field/web
npm install
npm run build -- --mode development
popd
