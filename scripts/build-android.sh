#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Install cargo-ndk and the four Rust Android targets in CI; NDK must be selected.
: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to the installed Android NDK}"
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 -o generated/android/jniLibs build --locked --release -p viptv-core --features native --lib -j 2
