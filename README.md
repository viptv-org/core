# viptv shared core

Actions delivery: main pushes and manual builds produce sideloading artifacts
(Android universal APK; desktop Windows/Linux installers; Roku ZIP; TV WGT/IPK).
Other repositories have no Actions workflows. Local checks remain; previous
CI/release-publication descriptions below are historical. No automatic deploys.

Crux Rust application behavior and API normalization for browser, Android and Tauri. The product contract is [SHARED_CORE.md](https://github.com/viptv-org/design/blob/c58c9b91827a39442462797602bded35a83a9f64/SHARED_CORE.md); this repository owns the implementation. Roku stays independent.

`crates/viptv-core` owns startup/session/profile state and normalization of backend identity, catalog, media, source and playback payloads. The Crux model requests HTTP, storage and rendering effects. The same library compiles to native code and browser WASM. Rust DTOs generate Kotlin/TypeScript interfaces and kotlinx.serialization JSON codecs in `generated/kotlin-wire`; `generated/typescript/wire.ts` describes the actual JSON bridge protocol. Native Kotlin calls use the generated UniFFI bridge, while browser JavaScript uses the generated wasm-bindgen bridge.

`packages/runtime` owns the shared TypeScript effect dispatcher and bounded HTTP execution. Browser fetch and Tauri native plugin fetch implement the same HTTP effect. Tauri loads the native Rust core, exposes its three commands, and uses `@tauri-apps/plugin-http` for network work; see `adapters/tauri`. Android uses native Rust and OkHttp, including PATCH; see `adapters/android`. Platform storage must use the appropriate credential store. Actual codecs and rendering remain platform responsibilities.

`crates/viptv-core::vizio` ports the MIT-licensed `get-air/vizio` SmartCast protocol into Rust. `SmartCastBridge` is a serialized native workflow for Android mobile and Tauri desktop: platform code executes one exact-TV-origin HTTPS request at a time and returns status/body, while Rust owns pairing-token adoption, input freshness, setting validation/retry and command payloads. Android adapter source includes a dedicated scoped-TLS OkHttp client and Android Keystore token store. Tauri adapter source includes a native reqwest client and OS keyring. SmartCast is not a playback backend and is not wired into browser, Android TV, Tizen, Vizio-hosted, or Roku applications.

## Playback error normalization

`normalize("apiError", ...)` maps stable backend `error_code` values to canonical,
user-safe messages. `normalize("playbackV2", ...)` uses the same mapping for failed
and expired leases, so HTTP refusal and asynchronous preparation failure retain
one explanation across native and WASM consumers. Gateway access/scope, capacity,
cleanup, startup timeout, processing, protocol and connectivity failures stay
separate from provider limits, source access/configuration and unsupported delivery.
A bare HTTP 429 is rate limiting, not evidence of a provider connection limit.

Known codes take precedence over upstream text. Terminal leases never expose raw
error text or delivery credentials; unknown codes use a safe fallback. HTTP errors
retain the existing bounded/redacted text fallback for compatibility. The output
contracts remain `message`/`code` for API errors and `error`/`errorCode` for leases.
These messages do not select another source or prescribe automatic retry. Native
player/decoder facts still belong to platform adapters; core does not infer their
cause from backend status codes.

Native adapters can report only a closed observed fact through
`normalize("nativeTorrent", {"operation":"failure","reason":"native_payload_limit"})`.
The result uses the same canonical `code`/`message` shape as `apiError`; shared
Rust owns exact copy for deadlines, aggregate cache budget, storage/cache,
metadata/file mismatch, grant expiry, connectivity and codec support. Unknown
exceptions map to the explicit `native_playback_failed` fact in the adapter.
Unknown reason codes, duplicate/extra fields, diagnostics and inputs over 512
UTF-8 bytes are rejected. No raw exception, URL, hash or path is accepted.
Metadata timeout requires that engine observation; total acquisition timeout
does not assert absent peers. Existing recovery decisions remain separate.

`tests/playback-error-vectors.json` supplies the canonical cases to the native
`playback_errors` integration tests and the actual WASM suite. Consumer adoption
requires the same immutable core revision and rebuilt native/WASM artifacts.

## Playback protocol foundation

The closed `playbackProtocolV2` raw-text normalizer and bodyless protocol/request-ID
cancellation operations are documented in [docs/playback-protocol.md](docs/playback-protocol.md).
The private native grant bridge and shared admission/recovery decisions are
documented there. Capability activation and platform/backend effects remain
separate qualification work; TV-web never advertises native delivery.

## Build and adoption

BE-002 retires the application-facing anonymous addon/provider bridge. Core
native/WASM exports no longer plan anonymous discovery or construct provider
media URLs. The standalone `crates/viptv-provider` parser/planner remains for
backend reuse; normalization, authenticated session/v2 wire and SmartCast remain.
Consumers must adopt the regenerated native bindings and WASM together. See
`scripts/test-v2-retirement-parity.mjs` for baseline native/actual-WASM comparison.

Use a current Rust toolchain and the WASM target. Run `cargo run -p viptv-typegen` to generate protocol types, `bash scripts/build-native-bindings.sh` for native Kotlin bindings, and `bash scripts/build-wasm.sh` with the matching wasm-bindgen CLI to build the browser artifact. Cargo.lock pins the dependency graph. This repository has no hosted CI and publishes no artifacts: run the generators and the local checks in VALIDATION.md, then commit the regenerated output with the source change.

Consumers adopt an immutable core commit together with its generated bindings and runtime. TV-web imports artifacts using `node scripts/core-sync.mjs sync ../core`, records CORE_REF and checks their hashes on build. Update Rust data handling here and regenerate once; consumer applications then adopt and rebuild. Installed apps still require an app update.

## WASI / BrightScript experiment

`crates/viptv-core/examples/wasi.rs` exposes the real Crux core through a persistent
JSON-lines process. One request produces one flushed response; the instance and
pending effects remain alive between lines. Run `bash scripts/build-brightscript.sh`
with the patched `wasm2brs` checkout beside this repository, or set `WASM2BRS_ROOT`.
That checkout owns the translator/runtime and writes converted files under `out/`.
For native bridge development, use `cargo run --release -p viptv-core --example wasi`.

Requests are JSON objects with `op`: `normalize` takes `kind`, `input`, and optional
`origin`; `vizio_platform_support` takes `platform`; `update` takes an `event` object
or enum string; `resolve` takes an effect `id` and `result`; `view` returns the view
model. `process_event` and `handle_response` alias `update` and `resolve`. `reset`
starts a fresh instance and returns its view. Responses are `{ "ok": true,
"result": ... }` or `{ "ok": false, "error": "..." }`. Effects retain the existing
generated JSON contract; the platform performs storage/HTTP and resolves their IDs.
Each input line is capped at 12 MiB (including JSON byte-array overhead); oversized
input returns an error and closes the process. Invalid JSON returns an error and
allows the next request.

After building, `python3 scripts/test-wasi.py ../wasm2brs/tools/wasmtime
../wasm2brs/artifacts/viptv-core-wasi.wasm` checks the existing shared startup vectors,
normalization, malformed inputs and warm latency. This is an experimental bridge;
it does not change the production Roku application's adoption status. See the
translator's `HANDOFF.md` for generated BrightScript and device validation.
The full conversion currently produces about 14 MB of BrightScript. The measured
Roku WASM/WASI initialization is about 296 ms; warm normalization of ten catalogs
takes about 431 ms. A 100-catalog checkpoint took 4,256 ms, so large batches still
need to stay off the interactive path. Keep it off animation/input hot paths; native/WASI host latency
is not representative of BrightScript execution. For a repeatable breakdown of
Rust work, run `cargo run --release -p viptv-core --example normalization_profile`.

## Migration state

Android and TV-web adopt the native and WASM library respectively. Both use shared response normalization and presentation/policy outputs; the Crux session model governs restoration. Canonical artwork roles, continuation, source selection and request/response handling live in Rust. UI navigation, input/focus, device storage/network execution and player lifecycle stay in platform shells. Refer to consumer validation records for exact coverage rather than treating a binding build as adoption.

Android vendors a hash-checked committed source snapshot and generated Kotlin; its own `scripts/prepare-core.sh` builds the host test library and three Android ABIs with cargo-ndk, locally or in the Android repository's build workflow. TV-web vendors the WASM module, generated TypeScript and shared effect runtime. Both record CORE_REF. Edit the owning Rust source here, regenerate, commit, and explicitly sync the same immutable revision into each consumer; do not edit vendor copies. This reduces duplicate rules, but generated code and reproducible source snapshots can increase checkout size.

Tauri's native command and native-fetch adapters are available and tested through their injected ports. No installed desktop application migration is claimed in this Android/web delivery.

Runtime tests use injected fetch/native command ports. Native Rust and WASM behavior tests are distinct from physical TV, Android and installed Tauri testing. See VALIDATION.md for the measured checkpoint.

## Shared card contract

`normalize("cardPresentation", { item, context: "queue" | "catalog" })` accepts optional `failedImages: string[]` transport observations and returns generated `CardPresentation`: image and its role, title, subtitle, optional normalized progress, and activation intent/label. After a final image fetch/decode failure, renderers report the original selected URL in `failedImages` and request this projection again; Rust skips that candidate while preserving the existing role order and all actions/progress. Failure observations are local to the current card, not persisted provider metadata. Renderers must not choose alternate image fields or reconstruct these labels. Queue metadata is enriched through `enrichHome`, which matches the exact episode before adopting its still/title and preserves source, progress and previous-episode identity. Missing episode artwork can use a known landscape or an explicit empty state; a series portrait is never relabeled as an episode still. Platform adapters execute bounded metadata requests; Rust owns their merge.

## Hero edge pool

`normalize("heroEdgePool", { mediaType, genres, availableEdges })` returns
generated `HeroEdgePool { category, edges }` for the TV hero backdrop edge fade.
Any `Animation`/`Anime` genre selects `Anime` for `series` and `Animation`
otherwise. Else the first genre, in input order, whose pool has an available edge
supplies the category. Genre keys match ASCII case-insensitively because core
keeps provider genre text verbatim; the result names the canonical table key.
`edges` is that pool in table order, filtered to the renderer's shipped edges.
Without a usable pool it is every available edge except the baseline `linear`
scrim, and `category` is null unless the animation rule chose it. An empty list
means keep the baseline. The genre table mirrors the hero shader index
`genreEdges`; renderers keep shaders, timing and shuffle bags, keyed by category.
`tests/hero-edge-pool-vectors.json` drives the native tests and the actual-WASM
parity suite.

## License

Copyright (C) 2026 viptv contributors.

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; version 2 of the License. See [LICENSE](LICENSE). The playback adapters (`viptv-org/video`, `viptv-org/tauri-video-plugin`) and the Android repository remain under their existing MIT OR Apache-2.0 terms.
