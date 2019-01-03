#!/bin/bash

set -e

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test
cargo test --features=serialize

if [[ "$TRAVIS_RUST_VERSION" == "beta" ]] || [[ "$TRAVIS_RUST_VERSION" == "nightly" ]]; then
    rustup target add wasm32-unknown-unknown
    cargo install --git https://github.com/alexcrichton/wasm-gc || true

    $DIR/examples/tetris/wasm/build.sh --with-npm-install
    $DIR/examples/title/wasm/build.sh --with-npm-install
fi
