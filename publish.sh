#!/bin/bash

set -euxo pipefail

publish() {
    cargo publish --manifest-path render/Cargo.toml
    cargo publish --manifest-path input/Cargo.toml
    cargo publish --manifest-path storage/Cargo.toml
    cargo publish --manifest-path text/Cargo.toml
    cargo publish --manifest-path decorator/Cargo.toml
    cargo publish --manifest-path menu/Cargo.toml
    cargo publish --manifest-path prototty/Cargo.toml
    cargo publish --manifest-path file-storage/Cargo.toml
    cargo publish --manifest-path monolithic-storage/Cargo.toml
    cargo publish --manifest-path grid/Cargo.toml
    cargo publish --manifest-path unix/Cargo.toml
    cargo publish --manifest-path glutin/Cargo.toml
    cargo publish --manifest-path wasm-input/Cargo.toml
    cargo publish --manifest-path wasm-render/Cargo.toml
    cargo publish --manifest-path wasm-storage/Cargo.toml
    cargo publish --manifest-path wasm/Cargo.toml
}

read -r -p "Are you sure? " response
case "$response" in
    [yY][eE][sS])
        publish
        ;;
    *)
        echo "ok then"
        ;;
esac
