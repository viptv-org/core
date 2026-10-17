# Native metadata origins and opaque lookup identity

This experimental overlay adds six UniFFI methods while preserving every preexisting method signature:

- `plan_detail_for_origin(profile, kind, id, card_json)`
- `detail_page_for_origin(profile, kind, id, card_json, offset, limit)`
- `plan_sources_for_origin(profile, kind, id, card_json)`
- `plan_subtitles_for_origin(profile, kind, id, extras_json, card_json)`
- `remove_favorite_for_origin(profile, revision, card_json)`
- `library_membership(profile)`

The expected Card contains optional `metadata_origin`, whose Stremio value identifies the owned connector and its persisted incarnation token. The token survives ordinary configuration updates and changes when the connector is removed/reinstalled, even if its row ID is deliberately reused. The current profile revision remains the access/reply fence. The incarnation is a public identity nonce, not a provider credential.

The host stamps catalog/search/discovery/detail cards using captured current connector records. It does not accept a provider body's claimed origin. Detail jobs retain expected assertions; fresh metadata must agree with expected IMDb/parent/episode association. A contradiction invalidates older proofs/pages for that lookup target. Paging an episode outside the first page can verify it from the same complete bounded document rather than guess its season/episode or issue hundreds of HTTP requests.

Only intrinsically public IMDb lookup identities may delegate metadata across add-ons. A Search/catalog-only issuer must still be enabled, currently assigned and match its captured incarnation; the actual metadata producer must have Metadata capability. Both captured connection revisions and origins are fenced before a reply publishes. Returned detail/episode Cards adopt the actual metadata producer origin, preserve raw public lookup selectors and carry verified identity/parent assertions. Opaque IDs (including opaque IDs with posted IMDb hints) never delegate. An explicit public episode beyond the returned page is checked against the complete bounded producer document before any paged reply or source proof is accepted.

Sources/subtitles retain the metadata lookup origin and the producing stream/subtitle connector incarnation separately. Private opaque lookup IDs stay with their issuer. Shared public lookup IDs may use other assigned capable producers. Every grant validates profile scope, expiry and captured incarnations at projection/open. SDK mappings use current authoritative metadata rather than client-supplied external IDs. Existing selected opaque grants survive discovery cancellation; origin/profile changes still invalidate opens.

The private source/subtitle open replies also carry a captured media context: exact lookup kind/ID plus normalized full identity and parent assertions. Native compares it with the opened playback context before fetching subtitle bytes. Connector equality alone cannot bind subtitles: two titles from one add-on, or the same opaque lookup remapped to another canonical title, must remain distinct. Assertion order and duplicate IDs do not change the context.

Lookup provenance and persistence identity are separate. A verified canonical alias may share saved state, but the original lookup ID and its origin remain on the Card and playback record. A posted external-ID hint cannot mint a shared key. When fresh proof is absent, Store writes remain scoped and non-aliasing, preserving any exact existing owned key. A player updating an already-owned record cannot accidentally transfer its progress to a later mapping. Archived owned favorites remain removable after provider disable/delete.

Membership comes directly from Store and includes canonical identities, assertion-bearing scoped lookup identities and explicitly ambiguous legacy identities. KMP must compare scoped origin/lookup/returned assertions and must not infer a canonical alias from preview claims. A legacy row without origin is not silently assigned to today's connector.

Native has no Simkl producer: reserved `simkl:*` lookup IDs are rejected in Stremio projection/planning and native persistence. Stremio external Simkl claims are removed. Canonical episode numeric spelling is exact; zero-padded lookup segments remain opaque unless verified metadata establishes a canonical episode alias. Raw lookup IDs are never rewritten for provider requests.

The app has adopted this origin overlay with Rust-generated wire/UniFFI bindings. Linux desktop and all three configured Android packaging libraries were rebuilt from the exact adopted source. The immutable submodule base remains unchanged; the uncommitted overlay is recorded in `docs/source-metadata-overlay.json`, and app packaging evidence is in `app/docs/evidence/native-origin-build-2026-10-15.json`. The final shared Kotlin/native integration gate is recorded there separately from compilation. No physical device or live service was changed.
