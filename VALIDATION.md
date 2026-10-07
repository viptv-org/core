# Tagged watched completion and active rewatch projections — 2026-10-04

The `media` DTO now projects the backend's optional `resume_active`,
`watch_date_known`, and `completion_only` booleans as typed camelCase facts on
movies, episodes and previous episodes. Missing flags remain absent. Episode
history merge retains these facts. `initialEpisode` ignores completion-only
sorting dates as activity, prefers the most recent real active resume even when
`watched=true`, and otherwise retains completed-to-next navigation and an
unwatched metadata fallback. Home enrichment retains the exact occurrence's
watched/resume facts; queue and hero actions prioritize active rewatch over
`next` without inventing completion dates or modifying the backend import policy.

New native regressions failed 2/3 before the fix (missing DTO/merged facts), then
passed 3/3. `cargo fmt --all -- --check`, `cargo test --workspace --locked`
(70 tests), `cargo test --workspace --all-features --locked` (70 tests),
`cargo clippy --workspace --all-targets --locked -- -D warnings`, and the
actual WASM suite (`node scripts/test-wasm.mjs`, including import facts and
active rewatch vectors) pass. `cargo run --locked -p viptv-typegen` regenerates
wire Kotlin/TypeScript; native UniFFI generation from the Windows DLL changes
only template trailing whitespace, so the existing checked-in UniFFI bridge
stays unchanged. `cargo build --locked -p viptv-core --release --target
wasm32-unknown-unknown` and wasm-bindgen CLI 0.2.92 regenerate the real binary.
No consumer
adoption, server repair, deployment or hardware claim follows from these checks.

# Successful response validation keeps HTTP status — 2026-10-02

Malformed successful identity/token responses now retain the actual HTTP status
in the session error view. A received HTTP200 with an empty required account name
was previously projected with no status and misclassified by consumers as a
connection refusal. The public CoreBridge regression reproduces null before the
fix and200 afterward while retaining saved-session retry.

Formatting, strict workspace/all-targets Clippy,67 native workspace tests and
the actual WASM suite pass. Typegen/native binding output stays byte-identical;
the WASM binary is regenerated. This is a response-classification change, not an
identity-validation relaxation, transport policy change or native playback claim.

# Dead maximumHeight session field removed — 2026-09-30

Playback normalization no longer maps the retired profile quality preference
onto a `maximumHeight` session field, and `PlaybackSession` drops it (design
BACKEND_V2.md: maximum-quality feature removed, actual device limits retained).
No consumer reads it; the video controller already dropped it from its session
view. Preferences still normalize `quality` because the backend stores it.

Typegen, native Kotlin bindings and the WASM build were regenerated: the
TypeScript wire/types, Kotlin wire/types and WASM binary changed; UniFFI Kotlin
and wasm-bindgen glue are byte-identical. Formatting, strict default and
all-feature workspace Clippy, 66 workspace tests (default and all features) and
the actual WASM suite passed. No consumer or device qualification follows.

# Source quality reference and policy hardening — 2026-09-30

With no reported device maxHeight, sourceMatch/continuationSource now rank "best"
against the highest resolution the device is likely to play among the compared
sources (optional sourceMatch `candidates`; a lone source is judged on its own).
A reported maxHeight remains the reference and no quality cap returns. Policy
regexes compile once per process; two FFI-reachable unwraps were removed;
viptv-provider (backend-only) moved to edition 2024.

Formatting, strict default/all-feature workspace Clippy, all 66 workspace tests
(default and all features), the actual WASM suite, nine runtime tests plus strict
TypeScript, and the 35-operation retirement parity against `fba95c8` passed. A
1,912-case policy corpus is byte-identical to the previous WASM except no-limit
sourceMatch results. Typegen, native Kotlin and wasm-bindgen glue regenerate
byte-identically; only the WASM binary changed. The backend suite (215 tests)
passed against the updated provider crate in a scratch copy; the backend's
vendored copy is not re-synced here. No consumer or device qualification follows.

# Application-facing local provider retirement — 2026-09-30

Core `8ae9f81` removes the retired application provider/add-on bridge and feature
selection while retaining the standalone provider parser used by the backend.
The provided native/WASM generators were rerun; frozen v2 event/JSON declarations
remain byte-identical. Workspace default/all-feature tests, runtime tests, strict
Clippy and 35 baseline/candidate native plus actual WASM operations passed. See
[LOCAL_RETIREMENT.md](LOCAL_RETIREMENT.md) for exact evidence and boundaries.

Reviewed consumers now pin this source: TV-web `db9c5ab`, desktop `54854cf` and the
isolated Android handoff `75bbacf`. The user's Android checkout remains untouched.
These checks do not qualify physical-device media, installed Windows playback,
production migration or deployment.

# Reverse guide cursors and uncapped source matching — 2026-09-29

Live channel/category DTOs now carry nullable previousCursor alongside the next
cursor. Reverse tokens use the same bounded opaque syntax. Source matching no
longer combines decoder dimensions with the retired profile quality preference,
and unknown decoder height no longer invents a 1080p ceiling. Actual reported
decoder limits still reject unsupported sources.

All 64 workspace tests, strict Clippy, Kotlin/native generation, release WASM
and actual WASM contracts passed. TV-web and Android imported code revision
fba95c8f3ba00e97fbc460acc746d23a912795bc. Client guide adoption and physical 4K
playback remain separate acceptance gates.

# Raw live v2 contracts — 2026-09-29

Generated LiveCatalogPage/LiveCatalogCategories preserve account catalog and
snapshot generation, provider order, HTTP logos and opaque next cursors. No exact
total is fabricated. Canonical request operations handle raw channel/category
pages, exact live source and guide reads, rejecting offset/US/family filters,
oversized pages/tokens and invalid collection choices. Favorites/Recent remain
explicit personal subsets, not whole-playlist indexing. Exact live source decoding
accepts an opaque IPTV card and rejects media URL/header authority at this seam.

All 63 workspace tests, strict Clippy, Kotlin/native generation, release WASM and
actual WASM contracts passed. Four new native fixtures cover default/cursor,
empty/malformed/legacy pages, HTTP logos, 4K-neutral data and private source cards.
Consumer adoption and activation are recorded separately; this does not claim
ordinary guide migration, physical playback or a production deployment.

# Backend v2 player-option mapping — 2026-09-29

PlaybackV2Request now includes the gateway's closed conversion enum, track
language preferences and subtitle-off choice. playbackV2Intent is the shared
bridge from measured player capabilities/options to this canonical request;
Tauri/HTML platform aliases, scoped audio/video conversion and explicit track
choices are normalized once. Reported 4K dimensions remain unchanged and profile
quality is excluded. Native direct lease normalization retains bounded language
preferences without restoring the removed quality cap. Contradictory subtitle
choices and malformed language tags fail validation.

All 59 workspace tests, strict Clippy, native/Kotlin generation, release WASM
and actual WASM contracts passed. Ordinary player activation and backend profile
preference integration remain open; this checkpoint supplies their shared
contract, not end-to-end playback or hardware qualification.

# Backend v2 playback lease contract — 2026-09-29

Generated PlaybackLease/PlaybackV2Request/PlaybackClient and closed platform,
lease-status and delivery-kind enums now describe the backend contract. The
playbackV2 normalizer preserves native HTTP direct URLs/validated source headers,
requires absolute HTTPS gateway delivery without attached credential headers,
keeps processing mode distinct from delivery kind, and exposes no session URL
for pending or terminal leases. Provider error text is replaced with safe shared
messages. Lease expiry is explicit Unix milliseconds; renewal is seconds.

Canonical request/start/status/heartbeat/stop operations validate the current
v2 wire contract. Real 4K decoder dimensions survive unchanged. Unknown legacy
options (including forceTranscode) are rejected, not silently discarded: backend
conversion/preference parity and client lifecycle adoption remain required.
This is a shared protocol checkpoint, not a completed playback cutover.

All 57 workspace tests, strict all-target Clippy, generated native Kotlin and
release WASM builds, and the actual WASM suite passed. Five new native fixtures
cover direct/gateway policy, safe failures, malformed envelopes and canonical
request serialization. Both consumers must pin this same committed revision.

# Backend v2 source discovery — 2026-09-29

Explicit sourcesV2/sourcesPollV2 request operations preserve exact movie/episode
identity. Generated Kotlin/TypeScript poll state carries bounded, deduplicated
source failures with closed human-readable messages; raw provider error text is
not displayed. Healthy sources remain available when another provider fails.
Legacy operations remain temporarily for live discovery pending its coordinated
catalog/playback cutover; this checkpoint does not complete that migration.

Type generation, native bindings, release WASM, all 52 workspace tests, strict
all-target Clippy and the actual WASM contract suite passed. Android and TV-web
must adopt this same immutable revision; application and device acceptance is
recorded separately in their repositories. HTTP provider support is unchanged.

# Native playback metadata follow-up — AND-036

Source display projections now include stable provider group keys and labels;
missing provider IDs no longer merge unrelated IPTV/addon sources. Native direct
playback preserves bounded, single-line upstream headers (including Referer)
alongside Cookie/User-Agent, while rejecting transport-owned Host/Connection
headers. Kotlin/TypeScript and WASM artifacts were regenerated from Rust. The
workspace unit/integration suites pass; consumer/device evidence remains separate.

# WASI ownership follow-up — 2026-09-22

Catalog construction now moves owned strings, arrays and sanitized raw metadata
into the output instead of serializing temporary values into a second tree.
The string bridge also retains its owned parsed catalog metadata in place;
borrowed callers share the same catalog-field constructor. Both sanitizer walks
use the same credential-key predicate. Typed DTO validation, exact raw numbers,
field omission, invalid-catalog isolation and request bounds remain unchanged.

All 47 Rust tests, formatting, strict core/all-target Clippy, and native/raw/
optimized/canonical WASI protocol checks passed. A permanent regression compares
216 catalog boundary/metadata combinations against the previous constructor.
The existing 1,035-document sanitizer corpus also exercises the new owned clean
path. An additional deterministic differential run compared 1,500 complete
catalog/catalogs/clean NDJSON responses against the saved pre-change executable;
all response bytes, including errors, matched.

Native ten-catalog normalization measured about 24 microseconds versus 29 before
this ownership change. This is a host measurement, not Roku latency. The isolated
serde_json-only speed-optimization experiment passed host checks and conversion
but initially failed real Roku compilation with a Label/Line NotFound error.
Removing unused generated labels allowed it to compile, but its 466 ms device
normalization remained slower than the selected size-optimized profile.

The final official translated build passed 32 exact device response comparisons,
including unsigned/signed 64-bit limits, the largest exactly representable
53-bit integer and negative zero, plus memory/compiler/WASI and both native
allocator stress suites. The numeric case exposed a pre-existing signed-zero
runtime decoder bug; the translator runtime fixed it and added bit-roundtrip
and memory-store/load regressions before the final pass.

With the combined translator/runtime changes, the final warm ten-catalog median
was 431 ms (434 ms maximum, 438 ms cold), versus 594 ms before this follow-up:
27.4% less latency. This is 68.7% below the original 1,377 ms checkpoint. The
hundred-catalog checkpoint passed at 4,256 ms warm and 4,382 ms cold, about 43 ms
per item; large batches still take seconds. View/update/storage/HTTP medians were
4/43/68/122 ms (HTTP maximum 213 ms). Actual WASM/WASI initialization was 30/266 ms,
296 ms total, excluding regression fixtures. The generated BrightScript totals
14,082,747 bytes. These are experimental channel measurements; the production
Roku application's adoption status is unchanged. Final artifact/device evidence
belongs to the translator's `HANDOFF.md`.

---

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
translator/runtime improvements and the checked native allocator enabled, all 31
device parity cases and allocator stress regressions passed. The final ten-catalog
median was 594 ms (44.7% lower latency, 1.81 times faster than the 1,075 ms
checkpoint), with a 601 ms cold call. View/update/storage/HTTP medians were
5/51/79/142.5 ms; the HTTP maximum was 247 ms. Unicode, nested typed metadata and
recursive sanitizer cases passed. The generated source totals 15,241,473 bytes
across 11 files. Timed initialization was 30 ms for WASM and 283 ms for WASI,
313 ms total; the earlier
3,945 ms number included regression fixtures and is not a comparable core-only
startup measurement. Final translator/runtime evidence is in its `HANDOFF.md`.
The separate 100-catalog device run also passed parity and allocator stress:
5,992 ms cold and 5,903 ms warm, about 59 ms per item. Scaling is approximately
linear; large batches still take seconds and must stay off the interactive path.

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
# Native torrent private transport and decisions — 2026-10-07

Design pin: `83d338b6ffc1fc5e7f14ad4059f6159b8ee84509`,
SRC-TORRENT-NATIVE-001. Closed private grant/input parsing, original UTF-8 byte
validation, canonical v1 metainfo/hash/file checks, scoped negotiation,
admission/idempotency, grant clocks/renewal and explicit recovery decisions live
in Rust. Dedicated native/WASM holders redact Debug/Display/toString and keep
private input out of ordinary session/source/presentation models. Native
advertisement remains gated by qualified Android facts; consumers do not gain
qualification or activation from adopting these interfaces.

Final source batch passed: 116 workspace tests with all features, strict
workspace/all-target/all-feature Clippy, formatting, type generation, native
UniFFI/Kotlin generation, release WASM generation and the full actual-WASM
suite. Its 410 native torrent vectors match the native Rust runner exactly,
alongside 90 protocol vectors, 37 shared-policy vectors and existing startup,
normalization and presentation regressions. Generation used wasm-bindgen CLI
0.2.92 and checks used Node 24; Cargo ran one job. Build scripts honor the
configured Cargo target directory so generation consumes the freshly compiled
artifact. Logs are local at `target/native-torrent-05/`.

Android and TV-web adoption/checks are recorded in their repositories. These
core checks execute no peer transport, network effects, Android device playback
or production deployment. JNI/artifact, complete backend integration and
physical/public-peer qualification remain separate tickets.
