# Playback preference upstream (backend#43) — 2026-10-27

`ProfilePreferences` carries the three playback choices the backend vendored by
hand in its engine snapshot (`53c3085`): `next_episode_button` (default on),
`skip_intros` (default off) and `match_frame_rate` (default on). All three are
serde-defaulted in `StoredPreferences`, so rows saved before this change decode
with the designed defaults, and the wire form now always projects them as
booleans. A new `tests/preferences.rs` case pins the defaults, the legacy-row
migration, the round trip of non-default choices and the rejection of
non-boolean values.

The engine contract is unchanged: `ProfilePreferences` is not registered with
typegen and crosses the native bridge as JSON (`profile_preferences`,
`update_profile_preferences`), so `cargo run -p viptv-typegen --bin
viptv-typegen`, `--bin engine` regenerate with no diff. No UniFFI or WASM
rebuild is needed (the browser graph excludes `viptv-engine`).

Rust 1.98.0: `cargo test --workspace --all-features --no-fail-fast`: 329
passed, 1 failed, 4 ignored. The failure is the pre-existing search-title
assertion (core#10), untouched here; `cargo test -p viptv-engine --features
runtime --test preferences`: 4 passed.

---

# Local engine avatar catalog (ADM-004) — 2026-10-26

Local engine profiles store the design avatar catalog: `ProfileConfiguration`
gains `avatar_style` (absent means `critters`) and a 1-based `avatar_choice`
(`None` until a catalog pick, as the backend projects `null`). Both are
serde-defaulted and optional on the wire, so stored rows and older payloads
still decode. `set_profile_avatar_choice` validates against `AVATAR_WORLDS`
(16 worlds, 624 avatars), which a test checks against the vendored
`tests/avatar-catalog.json` (design `3884fb3`). Legacy ids stay accepted; a
catalog pick retires them and, as on backend `5b22518`, wins over a later
legacy write in `profile_cards`.

Rust 1.98.0: `cargo run -p viptv-typegen --bin viptv-typegen`, `--bin engine`
(also picks up the earlier `LiveQuery.page_size`) and
`scripts/build-native-bindings.sh` regenerated the committed outputs.
`scripts/build-wasm.sh` (stable 1.97.1, the toolchain with the wasm32 target,
wasm-bindgen 0.2.92) succeeds, but its output is not committed: the browser
graph excludes `viptv-engine` and the local engine is `native`-only.
`cargo test --workspace --all-features --no-fail-fast`: 328 passed, 1 failed,
4 ignored. The failure is the existing search-title assertion (core#10),
unrelated to avatars; the seven avatar/profile tests pass.

---

# Certification projection and canonical server snapshot — 2026-10-23

Stremio title and episode Cards now preserve bounded declared certification
text (`contentRating`, then `certification`, `ageRating`, `mpaaRating`). Missing
or malformed values do not become inferred age labels. The existing optional
CardMetadata field and binding ABI are unchanged; no generated schema change or
WASM distribution build is needed for this server-engine adoption.

The canonical Home request cap is 16, preserving the behavior already present
in the backend export. Its dedicated regression covers the boundary with 20
requested rows. The server's stale nested source copies and manifest drift are
handled by its immutable export/check workflow, not manual vendor edits.

`cargo test -p viptv-engine --all-features -j 2 -- --test-threads=1`:
114 passed, zero failed, one existing ignored network test. The two certification
regressions failed before the change. Host parental policy remains a separate
consumer of verified provider evidence; these presentation strings are not
permission grants.

---

# Best match release facts and ranking (SRC-BEST-MATCH-001 phase 1) — 2026-10-22

`stremio::source_release_facts` derives `release_type`, `three_d`,
`dynamic_range`, `audio_flags` and `seeders` from declared fields first, then
PTT-equivalent release-name rules (filename, title, name, description lines;
Torrentio's seeders/size line only supplies `👤 N`). `stream_delivery` reads
the transport only. `SourceCandidate` carries the facts plus `delivery`; each
is omitted when unknown, so earlier payloads parse unchanged. Dual/multi/dub
markers are flags, never audio languages; `Multi Subs` no longer hides the flags
on Torrentio's language line, and VOSTFR/SUBFRENCH French is not audio.

`rank_sources(candidates, preferences, original_language, device_max_height)`
orders: excluded (camera, 3D, torrent with fewer than 5 known seeders), language
tier, cap class with cap = min(profile cap, device height), resolution (highest
first within the cap, closest first over it), release type, `hdr_preference`,
delivery and seeders, then input order. Original-language metadata is not
plumbed yet; hosts pass None. `ProfilePreferences` adds `hdr_preference`
(`any`/`prefer`/`avoid`) and `subtitle_mode` (`always`/`foreign`/`never`; rows
without it derive it from `subtitles_enabled`, a sent mode decides
`subtitles_enabled`). The native bridge adds
`rankSourcesForDevice(profile, sourcesJson, deviceMaxHeight: UInt?)`;
`rankSources` is unchanged. Engine Kotlin/TypeScript contracts and UniFFI Kotlin
bindings were regenerated; the browser WASM was not rebuilt (no client uses
ranking through it, and wasm-bindgen 0.2.92 is not installed here).

`cargo test --workspace --all-features` (Rust 1.98.0): 319 passed, 0 failed.

# Stremio subtitle execution — 2026-10-12

Shared subtitle planning negotiates declared resource types and ID prefixes,
encodes actual optional filename/hash/size evidence once, and bounds/normalizes
subtitle responses. Candidate IDs are host-generated; public snapshots contain
language/provider labels and per-provider Ready/Empty/Failed states, never URLs.
The native bridge plans, projects and opens subtitles through the selected
profile's enabled subtitle assignments. Choices live in memory for ten minutes
and become unusable after profile configuration changes. Additive native methods
are `plan_subtitles`, `project_subtitles` and `open_subtitle`; wire models were
regenerated. UniFFI Kotlin bindings were regenerated from the adopted native library;
Android assembly and desktop integration tests passed in the KMP app.

Three subtitle contract tests and three native addon/bridge tests pass. Shared
UTF-8 SRT-to-WebVTT normalization preserves native VTT and rejects binary/archive,
unsupported encoding, malformed timing and unbounded content. Native player
attachment and external-provider compatibility still require host qualification.

# Local Home catalog cache — 2026-10-12

The native engine persists successful catalog card projections in SQLite for
five minutes. Its next Home plan requests only missing/expired rows; prepared
Home includes cached rows and featured artwork alongside fresh watch history.
Cached catalogs are isolated by connection/account/profile/configuration
revision, capped at twelve rows of twenty-four cards, and exclude watch progress.
Request URLs and raw provider responses are not persisted. Search never populates
the Home cache; failures remain retryable and are not cached. Profile deletion
removes cached data. Existing UniFFI signatures and presentation types are unchanged.

`cargo test -p viptv-engine --features runtime` passes sixteen tests, including
expiry, isolation, revision rejection, bounds and deletion. The two native addon
bridge tests pass with `cargo test -p viptv-core --features native --lib
local_engine::addons::tests`; they cover zero warm requests, restart persistence,
mixed cached/failed row ordering and search separation. Real provider latency
and incremental publication remain unqualified.

Bounded asynchronous Home assembly now marks deadline-expired rows Failed with
retry copy and returns a complete snapshot, because its remaining futures are
cancelled. Pending no longer implies a continuation that does not exist.
