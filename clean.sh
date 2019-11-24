#!/bin/bash

set -euxo pipefail

cargo clean

find -name node_modules -exec rm -rf {} \; || true
find -name package-lock.json -delete || true
find -name dist -exec rm -rf {} \; || true
find -name pkg -exec rm -rf {} \; || true
find -name target -exec rm -rf {} \; || true
