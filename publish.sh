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
    publish_single menu
    publish_single event-routine
    publish_single app
    publish_single prototty
    publish_single audio
    publish_single native-audio
    publish_single storage
    publish_single file-storage
    publish_single web
    publish_single ansi-terminal
    publish_single graphical
    publish_single graphical_common
    publish_single graphical_gfx

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
