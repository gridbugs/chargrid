#!/bin/sh
set -e
TARGET=--vulkan
OPTIMIZATION=--optimization-zero
DEBUG=--debug
shader-translator --vertex $TARGET $OPTIMIZATION $DEBUG < src/shader.vert > src/shader.vert.spv
shader-translator --fragment $TARGET $OPTIMIZATION $DEBUG < src/shader.frag > src/shader.frag.spv
