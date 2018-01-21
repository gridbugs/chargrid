#!/bin/bash

set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test --manifest-path $DIR/prototty/Cargo.toml
cargo test --manifest-path $DIR/unix/Cargo.toml
cargo test --manifest-path $DIR/unix_border/Cargo.toml
cargo test --manifest-path $DIR/wasm/Cargo.toml
cargo test --manifest-path $DIR/glutin/Cargo.toml
