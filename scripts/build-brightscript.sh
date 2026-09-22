#!/usr/bin/env bash
set -euo pipefail
CORE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM2BRS_ROOT="${WASM2BRS_ROOT:-$CORE_ROOT/../wasm2brs}"
if [[ ! -f "$WASM2BRS_ROOT/handoff.sh" ]]; then
  echo "Set WASM2BRS_ROOT to the patched wasm2brs checkout (see README)." >&2
  exit 1
fi
bash "$WASM2BRS_ROOT/handoff.sh" build-core
bash "$WASM2BRS_ROOT/handoff.sh" pipeline
