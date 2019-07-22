#!/bin/bash

set -euxo pipefail

cargo clean

find -name node_modules -exec rm -rf {} \; || true
find -name package-lock.json -delete || true
find -name dist -exec rm -rf {} \; || true
find -name wasm_out -exec rm -rf {} \; || true
