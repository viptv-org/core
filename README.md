# viptv shared core

Crux Rust application behavior and API normalization for browser, Android and Tauri. The product contract is [SHARED_CORE.md](https://github.com/viptv-org/design/blob/12e2a3a2a4ac62310c0597f4e08257e7d9c87300/SHARED_CORE.md); this repository owns the implementation. Roku stays independent.

`crates/viptv-core` owns startup/session/profile state and normalization of backend identity, catalog, media, source and playback payloads. The Crux model requests HTTP, storage and rendering effects. The same library compiles to native code and browser WASM. Rust DTOs generate Kotlin/TypeScript interfaces; `generated/typescript/wire.ts` describes the actual JSON bridge protocol. Native Kotlin calls use the generated UniFFI bridge, while browser JavaScript uses the generated wasm-bindgen bridge.

`packages/runtime` owns the shared TypeScript effect dispatcher and bounded HTTP execution. Browser fetch and Tauri native plugin fetch implement the same HTTP effect. Tauri loads the native Rust core, exposes its three commands, and uses `@tauri-apps/plugin-http` for network work; see `adapters/tauri`. Android uses native Rust and OkHttp, including PATCH; see `adapters/android`. Platform storage must use the appropriate credential store. Actual codecs and rendering remain platform responsibilities.

## Build and adoption

Use a current Rust toolchain and the WASM target. Run `cargo run -p viptv-typegen` to generate protocol types, `bash scripts/build-native-bindings.sh` for native Kotlin bindings, and `bash scripts/build-wasm.sh` with the matching wasm-bindgen CLI to build the browser artifact. Cargo.lock pins the dependency graph. CI builds native/WASM interfaces and publishes revision-named artifacts.

Consumers adopt an immutable core commit together with its generated bindings and runtime. TV-web imports artifacts using `node scripts/core-sync.mjs sync ../core`, records CORE_REF and checks their hashes on build. Update Rust data handling here and regenerate once; consumer applications then adopt and rebuild. Installed apps still require an app update.

## Migration state

First adoption is TV-web's shared Rust response normalization and generated domain types. Its current screen/controller layer still owns navigation, pairing orchestration and playback coordination. The Crux session model and reusable effect driver are available for subsequent consumer adoption; existing Android and desktop apps are not claimed migrated merely because their adapters exist. This first slice does not share every application rule yet.

Runtime tests use injected fetch/native command ports. Native Rust and WASM behavior tests are distinct from physical TV, Android and installed Tauri testing. See VALIDATION.md for the measured checkpoint.
