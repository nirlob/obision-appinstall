#!/bin/sh

# Parámetros desde meson
SRC_DIR="$1"
BUILD_DIR="$2"
OUTPUT="$3"
BUILDTYPE="$4"
APP_BIN="$5"

export CARGO_TARGET_DIR="$BUILD_DIR"/target
export CARGO_HOME="$BUILD_DIR"/cargo-home

if [ "$BUILDTYPE" = "release" ]
then
    echo RELEASE MODE
    cargo build --manifest-path "$SRC_DIR"/Cargo.toml --release
    cp "$CARGO_TARGET_DIR"/release/"$APP_BIN" "$OUTPUT"
else
    echo DEBUG MODE
    cargo build --manifest-path "$SRC_DIR"/Cargo.toml
    cp "$CARGO_TARGET_DIR"/debug/"$APP_BIN" "$OUTPUT"
fi
