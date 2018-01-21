#!/bin/bash

cargo test --manifest-path prototty/Cargo.toml
cargo test --manifest-path grid/Cargo.toml
cargo test --manifest-path unix/Cargo.toml
cargo test --manifest-path wasm/Cargo.toml
cargo test --manifest-path glutin/Cargo.toml
cargo test --manifest-path common/Cargo.toml

source examples/tetris/test.sh
source examples/title/test.sh
