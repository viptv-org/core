# Source metadata projection

`viptv-engine::stremio::SourcePresentation::from_stream` fills missing structured
presentation fields from bounded name, title, description and filename labels.
`source_quality_height`, `source_audio_languages` and `source_detail` expose the
same projection to server and native local hosts. No generated DTO changed.

Structured metadata wins. Quality otherwise prefers the source's reported quality
or name, then filename, then release title/description; captured cards contain a
real 720p label with an original 1080p filename, so a merged highest-number rule
would fabricate the rendition's reported quality. Unknown values stay unknown.

Codec/container/size/language values remain unverified provider reports. Release
labels never prove decoder support or selected audio. Subtitle-only mentions and
ambiguous two-letter release words do not become audio languages. Explicit unknown
language reports override title decorations. Delivery URLs and proxy headers never
supply technical display metadata; backend parsing follows private-value redaction.

Cached batches:

- `tests/source-metadata-vectors.json`: 802 synthetic cases across fields,
  quality/codec spellings, containers, audio codecs, language flags/tags, sizes,
  bounds, declaration precedence and absent/malformed values.
- `tests/source-metadata-captured.json`: 140 previously captured source cards.
  A closed allowlist retains technical tokens and punctuation; arbitrary names,
  media identities, source grants, hashes, account/provider identities and links
  are replaced or omitted. Golden quality labels and filename codecs/containers
  are checked independently. No network requests are needed to reuse this cache.

`cargo test -p viptv-engine --test source_metadata --test streams --locked --jobs 2`
checks all 942 cards in one batch and the actual local `read_streams` projection.
Measured batch: 333ms; three stream contracts: 10ms (host debug test binary,
compilation excluded). Backend tests additionally check the redacted public-card
projection (942 cards in 285ms, plus private-reflection regression). This is metadata acceptance, not stream or physical-device playback.

This work is an uncommitted experimental overlay. Native `app/native/core` and
backend `server/shared/engine` contain matching source, while the immutable core
pin stays at its base revision. Native overlay hashes are recorded in
`app/native/core/docs/source-metadata-overlay.json`. The corrected source was
rebuilt into local Linux desktop and three Android ABI packaging libraries;
`app/docs/evidence/native-engine-source-metadata-search-2026-10-15.json` records
source and artifact hashes. No installed app, Wasm output, running service or
physical device was changed by this source/build work.

Unified text Search now uses every enabled, profile-assigned Search integration
in the shared registry. Metadata and Discovery continue using the configured
browsing plugin. Native Search retains each addon connection ID in its request,
row and card projection; same-named providers and identical media IDs do not lose
provenance. Three registry tests cover defaults, scope/capability enablement and
local/server restrictions; a native planner regression covers per-addon groups.

Explicit provider language tags use the existing `preferences::language_tag`
validation and preserve unknown valid languages, scripts and regions (including
`cs`, `el`, `he`, `th`, `en-US`, `pt-BR`, `zh-Hant-TW`). Only known ISO3 aliases
are normalized; the finite vocabulary applies only to inferred release hints.
Explicit `und` remains `und`, without being replaced by an English label hint.
Known human language names are matched as whole values before tag validation,
including Portuguese and Ukrainian; arbitrary language prose is not tokenized.
