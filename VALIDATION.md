# WASI performance follow-up — 2026-09-22

Profiling exposed failed untagged-enum retries while validating every raw metadata
value, and fifteen complete credential-substring searches for every metadata key.
The JSON visitor now dispatches by the actual JSON type, and the sanitizer scans
each normalized key once while borrowing ordinary lowercase ASCII keys. The
schema, typed validation, credential filtering and request size limits remain.

All 46 Rust tests, formatting, strict core/all-target Clippy and native/raw/
optimized/canonical WASI bridge checks pass. A deterministic parity test compares
1,035 documents against independent copies of the old implementations, including
recursive objects/arrays, integer limits, extreme floats, Unicode and escaping,
mixed-case/separated credential substrings, and ten malformed inputs. Serialized
outputs and malformed JSON error strings match exactly.

On the development Roku, the visitor alone reduced the warm ten-catalog median
from 1,075 ms to 772 ms. Adding the sanitizer reduced it to 660 ms. With the
translator/runtime improvements enabled, all 30 device parity cases passed and
the median was 650 ms (1.65 times faster than the previous checkpoint). Timed
initialization was 30 ms for WASM and 284 ms for WASI, 314 ms total; the earlier
3,945 ms number included regression fixtures and is not a comparable core-only
startup measurement. Final translator/runtime evidence is in its `HANDOFF.md`.

The native phase profiler is reproducible with `cargo run --release -p viptv-core
--example normalization_profile`. On the same ten-catalog fixture, typed
validation fell from about 19 to 4 microseconds and total normalization from
about 52 to 30 microseconds. These host numbers are not Roku timings. Cargo
`opt-level = "z"` remains: experimental `s` and `3` variants exceeded Roku's
per-function label limit during conversion.

---

# WASI / BrightScript bridge — 2026-09-22

The persistent JSON-lines bridge reuses the existing `CoreBridge` and normalizer;
no application behavior or generated client protocol changed. Native, raw WASI,
optimized WASI and the canonical wasm2brs input all passed the same three startup
vectors, effect resolution, reset, operation aliases, malformed-input recovery,
normalization boundaries and bounded rejection of requests over 12 MiB.

Warm end-to-end normalization of 32 catalogs (30 requests, including Python pipe
and JSON overhead) measured median/p95: native 0.282/0.398 ms, raw WASI
0.491/0.607 ms, optimized WASI 0.500/0.584 ms, canonical WASI 0.501/0.571 ms.
These measurements use the host's Wasmtime runtime, not a Roku interpreter.
All 43 Rust tests, workspace formatting, strict core/all-target Clippy, and diff
whitespace checks passed after implementation. The translator's `HANDOFF.md`
records BrightScript conversion and device results separately; this checkpoint
does not adopt the bridge into the production Roku application.

The performance follow-up borrows nested request JSON with `RawValue`, buffers
each response before flushing it, and deserializes typed validation directly
from the normalized value instead of cloning the whole JSON tree. It preserves
typed validation and the normalizer's 2 MiB input cap. All four protocol phases,
43 Rust tests, formatting and strict Clippy passed again; added checks cover
explicit null input, escaped JSON keys and the 2 MiB normalization boundary.
After this change, host median/p95 for the same 32 catalogs was native
0.271/0.366 ms, raw WASI 0.507/1.291 ms, optimized WASI 0.370/0.446 ms and
canonical WASI 0.348/0.471 ms.

The complete translated program passed 29 exact-response comparisons on the
development Roku, including all shared startup vectors and a Japanese/accented/
emoji catalog, plus compiler/memory regressions. Measured initialization was
3,945 ms; median view/update/storage/HTTP calls were 6/59/92/171.5 ms.
Ten-catalog normalization improved from 1,377 ms to 1,075 ms warm (22% faster),
with a 1,078 ms cold call. The Unicode case completed in 85 ms and the complete
channel run took 9,344 ms. These device measurements
include concurrent translator/runtime improvements; they cannot isolate the
effect of the Rust changes. This remains unsuitable for per-frame work, and the
production Roku application remains unchanged.

---

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

### Custom catalog response verification

Bounded, read-only AIOMetadata GETs verified the configured catalog namespaces independently from their returned media types: anime returned 25 series, anime.series search returned 7 series, anime.movie search returned 1 movie, and collection search returned 5 movies. The latter three manifests require search. Following an anime result through its exact series metadata path returned 65 videos; a collection result through its exact movie metadata path returned 6 videos. No token, provider URL or account data was recorded. These are metadata checks, not playback qualification.

The response contract retains returned movie/series identity and reports optional unsupportedCount for unknown media types instead of silently treating a nonempty unknown response as an empty catalog. Known rows in mixed responses remain available. Rust response fixtures cover all four observed namespace/type pairs.
