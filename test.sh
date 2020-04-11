#!/bin/bash

set -euxo pipefail

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo clean

cargo test --workspace
cargo test --manifest-path=chargrid/Cargo.toml --features=serialize
cargo test --manifest-path=examples/roguelike/graphical/Cargo.toml --no-default-features --features=chargrid_graphical_gfx

cargo clean

find -name node_modules -exec rm -rf {} \; || true
find -name package-lock.json -delete || true
find -name dist -exec rm -rf {} \; || true

pushd $DIR/examples/tetris/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/drag/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/pager/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/colour_picker/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/roguelike/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/colour_grid/web
npm install
npm run build -- --mode development
popd

pushd $DIR/examples/soundboard/web
npm install
npm run build -- --mode development
popd
