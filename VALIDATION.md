# Transparent title artwork — 2026-09-13

Rust media normalization and presentation now expose `titleLogo` separately from hero, poster and episode artwork. Explicit title/clear-logo aliases and on-demand provider `logo` metadata are supported; generic live-channel logos remain station artwork. Parent-series logos inherit into episodes without changing episode identity, title text or resume progress. Home metadata enrichment carries the normalized logo, and missing/blank logos retain text fallback.

Regenerated TypeScript declarations, Kotlin declarations/JSON codecs and the release WASM module from this source. Formatting, strict workspace/all-target Clippy, all 36 Rust tests, the actual WASM contract suite (including title-logo assertions), and strict runtime TypeScript checking passed. Cargo used one build job. No native-library/UniFFI regeneration was needed because its JSON bridge signatures did not change; no Gradle, emulator or physical-device check was run. Consumer adoption requires updating its immutable core pin and artifact snapshot.

---

# SmartCast native controller — 2026-09-13

Design pin: c58c9b91827a39442462797602bded35a83a9f64, CORE-003 in SHARED_CORE.md. Upstream protocol baseline: get-air/vizio@124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d.

The final shared-core batch passed formatting, strict workspace/all-target Clippy, 34 Rust tests, native UniFFI build/generation, release WASM compilation and parity checks, strict TypeScript checking, and all nine runtime tests. Ten SmartCast contracts cover immediate pairing-token adoption, the complete upstream remote-code table, Conjure launch payloads, case-insensitive status handling with unknown-status redaction, ASCII/request limits, bounded discovery, Android-mobile/Tauri-only support reporting, serialized commands, stale-response rejection, exact input matching with a fresh current-input hash, and one `HASHVAL_ERROR` refetch/retry.

Generated Kotlin, TypeScript and UniFFI interfaces include the native `SmartCastBridge`. Android mobile adapter source uses a dedicated exact-origin OkHttp client and Android Keystore AES-GCM token store. Tauri adapter source uses a dedicated no-redirect reqwest client, immediate cancellation and the OS keyring. These integration sources are compiled by their consuming applications; no local Gradle/emulator, installed Tauri application, or physical SmartCast pairing/launch test was run here. Browser, Android TV, Tizen, Vizio-hosted, and Roku SmartCast support is not claimed.

---

# Shared-core application adoption — 2026-09-13

Historical design pin: e821c297de2e81b47a6a1b22ed8aa0522cdd04e5, CORE-002 in SHARED_CORE.md.

The final implementation batch passed formatting, strict workspace/all-target Clippy, Rust contracts including profile-confirmation and source-presentation regressions, native UniFFI build/generation, WASM compilation, three native/WASM startup vectors and eight additional shared presentation/Android compatibility/request assertions. The runtime's nine tests and strict TypeScript check passed, including native Tauri command and fetch adapters through injected ports.

Coverage includes landscape hero versus portrait-only metadata, episode/progress identity, source/continuation policies, Android response compatibility, structured parent authorization status, restoration and stale effects. Generated Kotlin JSON codecs and native bindings are build inputs to the actual Android app, which performs its compilation/unit/APK checks on hosted CI. TV-web uses the actual WASM module and shared session driver. Their exact integration results are recorded in each consumer's TESTING.md; library results do not replace those checks.

No emulator, new physical Android/Tizen/Vizio playback qualification, or installed Tauri application test is claimed for this library batch. Tauri native transport and commands are delivered; a desktop application adoption remains separate.

---

Historical first checkpoint follows; its migration limitations describe that earlier revision.

# First shared-core checkpoint — 2026-09-13

- Crux 0.20.0 native library and browser WASM compile.
- 14 Rust contracts pass, including storage/session restoration, token refresh retaining profile identity, malformed/unsupported catalog isolation, source sanitization and media capability URL checks.
- `node scripts/test-wasm.mjs` passes three startup vectors also executed by the native Rust tests, plus domain boundary checks through the actual generated WASM artifact.
- Formatting and strict workspace/all-target Clippy pass. Native shared library and Kotlin UniFFI bindings generated successfully.
- Nine TypeScript runtime/transport tests pass: correlated effect dispatch, native Tauri command mapping, binary/status/header preservation, URL policy, redirects, cancellation, deadlines and body limits. The Tauri plugin is injected in these tests; they are not an installed desktop qualification.
- Kotlin HTTP adapter supports OkHttp PATCH and has Android instrumentation test source. No Gradle, emulator, physical Android or installed Tauri validation was run in this checkpoint.

TV-web adopts the Rust normalizers and generated domain types. Its own browser/playback evidence belongs in tv-web/TESTING.md. Existing Android/desktop applications have not yet adopted the core. Existing TV navigation/session orchestration still has remaining extraction work; this is a working first library release, not a completed all-platform migration.

Strict TypeScript validation also passed against the actual pinned Tauri HTTP plugin types; the runtime imports Rust-generated JSON wire types directly.

## 2026-09-14 shared card correction

Added generated CardPresentation for Kotlin and TypeScript and rebuilt WASM. Native workspace tests pass (39 tests), including exact episode still vs parent/neighbor image, missing-art fallback, source/progress preservation and catalog/live/queue intent. Consumer browser and Android validation is recorded in their repositories; no Android device or emulator claim.

Real populated Home exposed a2,305,123-byte enrichDetail bridge input: queue rows repeated the full episode catalog and metadata repeated it again in raw. A400-episode regression failed the2MiB bridge cap before correction and passes afterward. Shelf enrichment now retains the matched still rather than full episode lists; media raw omits already-typed episode arrays and synopsis fields. The2MiB input bound stays intact. Final native workspace:40 tests pass.

## Addon catalog namespaces — 2026-09-14

Production read-only manifest counts showed 78 catalogs, including 15 AIOMetadata catalogs in anime, collection, anime.series and anime.movie namespaces. The previous MediaKind restriction discarded those catalogs. Catalog now preserves its bounded exact addon-defined type and optional addon name; playable media keeps its separate typed contract. The targeted regression reproduced 1 of 5 fixture catalogs retained before the fix. Type generation and WASM regeneration accompany this revision. All 42 Rust unit/integration tests pass after the fix; consumer pins update together. Hardware qualification remains separate.
