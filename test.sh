#!/bin/bash

set -e

export RUSTFLAGS="--deny warnings"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test --manifest-path $DIR/prototty/Cargo.toml
cargo test --manifest-path $DIR/grid/Cargo.toml
cargo test --manifest-path $DIR/unix/Cargo.toml
cargo test --manifest-path $DIR/wasm/Cargo.toml
cargo test --manifest-path $DIR/glutin/Cargo.toml
cargo test --manifest-path $DIR/common/Cargo.toml
cargo test --manifest-path $DIR/file-storage/Cargo.toml

source examples/tetris/test.sh
source examples/title/test.sh
