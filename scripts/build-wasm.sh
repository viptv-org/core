#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked -p viptv-core --release --target wasm32-unknown-unknown
core_target_dir="${CARGO_TARGET_DIR:-target}"
wasm-bindgen --target web --out-dir generated/wasm --out-name viptv_core "$core_target_dir/wasm32-unknown-unknown/release/viptv_core.wasm"
