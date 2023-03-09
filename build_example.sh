#!/bin/sh

set -ex

# Build the core wasm binary that will become a component
cargo rustc -p "$@" --target wasm32-unknown-unknown --release --crate-type="cdylib"

# Translate the core wasm binary to a component
wasm-tools component new \
  target/wasm32-unknown-unknown/release/"$@.wasm" -o "target/$@.wasm"
