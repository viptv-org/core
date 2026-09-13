# viptv shared core

Crux Rust application behavior and API normalization for browser, Android and Tauri. The product contract is [SHARED_CORE.md](https://github.com/viptv-org/design/blob/c58c9b91827a39442462797602bded35a83a9f64/SHARED_CORE.md); this repository owns the implementation. Roku stays independent.

`crates/viptv-core` owns startup/session/profile state and normalization of backend identity, catalog, media, source and playback payloads. The Crux model requests HTTP, storage and rendering effects. The same library compiles to native code and browser WASM. Rust DTOs generate Kotlin/TypeScript interfaces and kotlinx.serialization JSON codecs in `generated/kotlin-wire`; `generated/typescript/wire.ts` describes the actual JSON bridge protocol. Native Kotlin calls use the generated UniFFI bridge, while browser JavaScript uses the generated wasm-bindgen bridge.

`packages/runtime` owns the shared TypeScript effect dispatcher and bounded HTTP execution. Browser fetch and Tauri native plugin fetch implement the same HTTP effect. Tauri loads the native Rust core, exposes its three commands, and uses `@tauri-apps/plugin-http` for network work; see `adapters/tauri`. Android uses native Rust and OkHttp, including PATCH; see `adapters/android`. Platform storage must use the appropriate credential store. Actual codecs and rendering remain platform responsibilities.

`crates/viptv-core::vizio` ports the MIT-licensed `get-air/vizio` SmartCast protocol into Rust. `SmartCastBridge` is a serialized native workflow for Android mobile and Tauri desktop: platform code executes one exact-TV-origin HTTPS request at a time and returns status/body, while Rust owns pairing-token adoption, input freshness, setting validation/retry and command payloads. Android adapter source includes a dedicated scoped-TLS OkHttp client and Android Keystore token store. Tauri adapter source includes a native reqwest client and OS keyring. SmartCast is not a playback backend and is not wired into browser, Android TV, Tizen, Vizio-hosted, or Roku applications.

## Build and adoption

Use a current Rust toolchain and the WASM target. Run `cargo run -p viptv-typegen` to generate protocol types, `bash scripts/build-native-bindings.sh` for native Kotlin bindings, and `bash scripts/build-wasm.sh` with the matching wasm-bindgen CLI to build the browser artifact. Cargo.lock pins the dependency graph. CI builds native/WASM interfaces and publishes revision-named artifacts.

Consumers adopt an immutable core commit together with its generated bindings and runtime. TV-web imports artifacts using `node scripts/core-sync.mjs sync ../core`, records CORE_REF and checks their hashes on build. Update Rust data handling here and regenerate once; consumer applications then adopt and rebuild. Installed apps still require an app update.

## Migration state

Android and TV-web adopt the native and WASM library respectively. Both use shared response normalization and presentation/policy outputs; the Crux session model governs restoration. Canonical artwork roles, continuation, source selection and request/response handling live in Rust. UI navigation, input/focus, device storage/network execution and player lifecycle stay in platform shells. Refer to consumer validation records for exact coverage rather than treating a binding build as adoption.

Android vendors a hash-checked committed source snapshot and generated Kotlin, then hosted CI builds the host test library and three Android ABIs with cargo-ndk. TV-web vendors the WASM module, generated TypeScript and shared effect runtime. Both record CORE_REF. Edit the owning Rust source here, regenerate, commit, and explicitly sync the same immutable revision into each consumer; do not edit vendor copies. This reduces duplicate rules, but generated code and reproducible source snapshots can increase checkout size.

Tauri's native command and native-fetch adapters are available and tested through their injected ports. No installed desktop application migration is claimed in this Android/web delivery.

Runtime tests use injected fetch/native command ports. Native Rust and WASM behavior tests are distinct from physical TV, Android and installed Tauri testing. See VALIDATION.md for the measured checkpoint.
