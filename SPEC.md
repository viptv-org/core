# Shared Crux core

Contract: https://github.com/viptv-org/design/blob/12e2a3a2a4ac62310c0597f4e08257e7d9c87300/SHARED_CORE.md

One Rust implementation owns shared parsing and session state for Android, browser and Tauri. The first delivery supplies actual Crux state transitions, native/WASM bridges, generated language interfaces and bounded platform HTTP adapters. Existing app adoption and remaining migration are listed in README.md. Roku is excluded.

Acceptance: native/WASM execution parity; generated interface checks; HTTP cancellation/binary/status/origin handling; startup restoration and catalog/playback normalization against backend shapes. TV playback, navigation and profile regression checks run after code completion. Hardware and full migration remain separately qualified.
