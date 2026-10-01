# Contributing to VIPTV core

Thanks for your interest. VIPTV is a multi-repository product; this repository owns the shared Rust application state, data normalization, generated bindings and platform adapters.

## Workflow

1. Read the pinned `DESIGN_REF` commit and [design/SHARED_CORE.md](https://github.com/viptv-org/design/blob/main/SHARED_CORE.md) before behavior changes; record proposed UX changes in design first.
2. Search this repository's GitHub Issues before opening a new one.
3. This workspace also owns the `viptv-provider` crate (`crates/viptv-provider`). Only the backend uses it, vendored into `server/provider` by its `scripts/sync-provider.sh`; the core crate and generated bindings do not depend on it. Change it here, then re-sync the backend.
4. Generated Kotlin/TypeScript/WASM interfaces come from Rust; never hand-edit `generated/` or consumers' `vendor/` trees.
5. Validate before pushing: `cargo fmt --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`, `cargo run --locked -p viptv-typegen`, `bash scripts/build-native-bindings.sh`, `bash scripts/build-wasm.sh`, `node scripts/test-wasm.mjs`, and the runtime package checks in `packages/runtime`.

## License

Contributions are licensed under the GNU General Public License v2.0 only (see [LICENSE](LICENSE)). By contributing you agree your work is licensed under it.
