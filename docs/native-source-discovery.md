# Native progressive discovery and metadata ownership

`LocalEngine::plan_sources` owns one active Stremio source operation per profile.
`project_sources_partial(operation,responses,finish)` appends completed providers
and returns cumulative candidates with stable opaque IDs. A provider's first
completion is final for that operation; replays cannot replace private context.
Unknown/duplicate providers reject the entire supplied batch before admission.
`project_sources` retains its original one-shot signature and ABI checksum.

`cancel_source_discovery(profile,operation)` closes pending work. It preserves
issued grants so selecting a source can cancel remaining HTTP work without
revoking the selected URL/headers. A stale cleanup cannot remove a replacement
operation. Profile/configuration revisions, deleted profiles, operation expiry
and grant expiry are still checked. Native operation and grant maps are bounded;
the existing 80-candidate Stremio budget reserves capacity for each planned addon.

The Kotlin native adapter fetches up to three addons concurrently, projects each
completion and publishes it before waiting for siblings. Cancellation is checked
before callbacks; cleanup captures operation ownership even if cancellation occurs
on the IO return hop. SDK and Stremio results merge without deleting each other's
grants. HTTP failures/timeouts remain provider-local completed results.

SDK polling clones a job before waiting for profile storage, eliminating the
jobs-map/storage lock inversion. SDK replacement cancels old pending jobs and
prunes only expired or revision-invalid grants. Previously issued SDK selections
remain usable through cancellation and replacement.

Metadata page snapshots, complete documents, enrichment and verified SDK cards are
keyed by profile, addon origin, media type and raw ID. The new
`detail_page_for_provider` cache lookup preserves an explicit producer. Disabled
origins cannot fall through to a sibling. Legacy origin-free cache APIs return no
cached result when multiple eligible origins exist. Legacy `stremio` library
aliases resolve only when one current metadata producer is unambiguous. SDK proof
requires the selected metadata addon to remain enabled and profile-assigned; client
mapping hints do not replace cached provider evidence or episode coordinates.

Generated UniFFI Kotlin adds three methods; all pre-existing method checksums are
unchanged. Source/binding adoption is recorded as an uncommitted overlay on the
existing immutable base pin, not as a new immutable revision. See native overlay
hashes and local packaging evidence under app/native/core/docs and app/docs/evidence.

Validation includes five source-operation regressions, three SDK/origin regressions
and existing native cache/enrichment/profile tests: 40 passed, one pre-existing
ignored public metadata capture. Kotlin tests use real Rust/SQLite/UniFFI with only
external addon HTTP replaced, including early usable grants, cancellation,
configuration changes, controller Back and two-provider cached episode pages.
This is native source/ownership acceptance; physical playback is separate.
