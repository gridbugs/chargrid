#!/bin/bash

cargo clean --manifest-path render/Cargo.toml
cargo clean --manifest-path input/Cargo.toml
cargo clean --manifest-path storage/Cargo.toml
cargo clean --manifest-path grid/Cargo.toml
cargo clean --manifest-path unix/Cargo.toml
cargo clean --manifest-path wasm/Cargo.toml
cargo clean --manifest-path glutin/Cargo.toml
cargo clean --manifest-path common/Cargo.toml
cargo clean --manifest-path file-storage/Cargo.toml
