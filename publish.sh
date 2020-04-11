#!/bin/bash

set -euxo pipefail

publish_single() {
    cargo publish --manifest-path $1/Cargo.toml
    sleep 10
}

publish() {
    publish_single render
    publish_single test-grid
    publish_single input
    publish_single text
    publish_single decorator
    publish_single app
    publish_single event-routine
    publish_single menu
    publish_single prototty
    publish_single web
    publish_single ansi-terminal
    publish_single graphical-common
    publish_single graphical-gfx
    publish_single graphical
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
