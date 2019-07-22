#!/bin/bash

set -euxo pipefail

publish_single() {
    cargo publish --manifest-path $1/Cargo.toml
    sleep 10
}

publish() {
    publish_single render
    publish_single input
    publish_single storage
    publish_single text
    publish_single decorator
    publish_single menu
    publish_single prototty
    publish_single file-storage
    publish_single monolithic-storage
    publish_single grid
    publish_single unix
    publish_single glutin
    publish_single wasm
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
