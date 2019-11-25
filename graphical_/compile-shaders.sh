#!/bin/bash
set -euxo pipefail
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

shader-translator -f < $DIR/src/shader.frag > $DIR/src/shader.frag.spv
shader-translator -v < $DIR/src/shader.vert > $DIR/src/shader.vert.spv
