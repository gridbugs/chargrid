#!/bin/bash

set -e

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test

rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/alexcrichton/wasm-gc

$DIR/examples/tetris/wasm/build.sh --with-npm-install
$DIR/examples/title/wasm/build.sh --with-npm-install
