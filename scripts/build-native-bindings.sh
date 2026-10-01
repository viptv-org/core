#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked -p viptv-core --features native --lib -j 2
cargo run --locked -p viptv-core --features bindgen --bin viptv-uniffi-bindgen -j 2 -- generate --library target/debug/libviptv_core.so --language kotlin --no-format --out-dir generated/native-kotlin
# UniFFI's no-format Kotlin template emits trailing spaces. Normalize them so
# clean-checkout regeneration is byte-for-byte stable and reviewable.
LC_ALL=C sed -i 's/[[:space:]]\+$//' generated/native-kotlin/uniffi/viptv_core/viptv_core.kt
