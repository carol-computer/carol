#!/bin/sh

set -ex

# Build the core wasm binary that will become a component
# XXX: set opt-level=z because default (3) hits some bug causing it to hang forver
# The bug is something like: https://github.com/rust-lang/rust/issues/91011
# ...and z creates smaller binaries which is not a bad choice
RUSTFLAGS="-C opt-level=z" cargo rustc -p "$@" --target wasm32-unknown-unknown --release --crate-type="cdylib"

# Translate the core wasm binary to a component
wasm-tools component new \
  target/wasm32-unknown-unknown/release/"$@.wasm" -o "target/$@.wasm"
