#!/bin/sh

export BUILD_DIR="$1"
export SRC_DIR="$2"
export OUTPUT="$3"
export BUILDTYPE="$4"
export APP_BIN="$5"

if [ "$BUILDTYPE" = "release" ]; then
    echo "RELEASE MODE"
    cargo build --manifest-path "$SRC_DIR"/Cargo.toml --release
    cp "$SRC_DIR"/target/release/"$APP_BIN" "$OUTPUT"
else
    echo "DEBUG MODE"
    cargo build --manifest-path "$SRC_DIR"/Cargo.toml
    cp "$SRC_DIR"/target/debug/"$APP_BIN" "$OUTPUT"
fi
