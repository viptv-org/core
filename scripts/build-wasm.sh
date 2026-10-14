#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
core_target_dir="${CARGO_TARGET_DIR:-target}"
core_bindgen="${VIPTV_WASM_BINDGEN:-wasm-bindgen}"
if [[ "$("$core_bindgen" --version)" != "wasm-bindgen 0.2.92" ]]; then
  echo 'Browser bridge requires wasm-bindgen-cli 0.2.92, matching browser/Cargo.lock.' >&2
  exit 1
fi
cargo build --locked --manifest-path browser/Cargo.toml --release --target wasm32-unknown-unknown --target-dir "$core_target_dir/browser"
"$core_bindgen" --target web --out-dir generated/wasm --out-name viptv_core "$core_target_dir/browser/wasm32-unknown-unknown/release/viptv_core.wasm"
