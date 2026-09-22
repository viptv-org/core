#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked -p viptv-core --release --target wasm32-unknown-unknown --features provider
wasm-bindgen --target web --out-dir generated/wasm --out-name viptv_core target/wasm32-unknown-unknown/release/viptv_core.wasm
