#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
core_target_dir="${CARGO_TARGET_DIR:-target}"
cargo build --locked -p viptv-core --features native --lib -j "${CARGO_BUILD_JOBS:-2}"
cargo run --locked -p viptv-core --features bindgen --bin viptv-uniffi-bindgen -j "${CARGO_BUILD_JOBS:-2}" -- generate --library "$core_target_dir/debug/libviptv_core.so" --language kotlin --no-format --out-dir generated/native-kotlin
# UniFFI's no-format Kotlin template emits trailing spaces. Normalize them so
# clean-checkout regeneration is byte-for-byte stable and reviewable.
LC_ALL=C sed -i 's/[[:space:]]\+$//' generated/native-kotlin/uniffi/viptv_core/viptv_core.kt
