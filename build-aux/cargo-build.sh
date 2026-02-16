#!/bin/bash
# Build script called by Meson to invoke Cargo

set -e

export CARGO_HOME="${CARGO_HOME:-$MESON_BUILD_ROOT/cargo-home}"

if [ "$1" = "release" ]; then
    CARGO_ARGS="--release"
    TARGET_DIR="release"
else
    CARGO_ARGS=""
    TARGET_DIR="debug"
fi

cargo build $CARGO_ARGS \
    --manifest-path "$MESON_SOURCE_ROOT/Cargo.toml" \
    --target-dir "$MESON_BUILD_ROOT/target"

cp "$MESON_BUILD_ROOT/target/$TARGET_DIR/$2" "$3"
