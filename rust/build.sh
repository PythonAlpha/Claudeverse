#!/bin/sh
# Builds the Rust search scorer into ../profile_search.wasm
# One-time setup:  rustup target add wasm32-unknown-unknown
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/profile_search.wasm ../profile_search.wasm
echo "Built ../profile_search.wasm"
