#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Explicit Chrome87-era ceiling; all newer proposals remain disabled.
wasm-tools validate --features=-all,mutable-global,saturating-float-to-int,sign-extension,multi-value,bulk-memory,floats generated/wasm/viptv_core_bg.wasm
printf '%s\n' 'PASS WASM feature ceiling: no reference-types, SIMD, GC, multi-memory, memory64, exceptions, tail calls or threads'
