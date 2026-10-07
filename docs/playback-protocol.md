# Playback protocol foundation

Normative contract: design `83d338b6ffc1fc5e7f14ad4059f6159b8ee84509`,
[SRC-TORRENT-NATIVE-001](https://github.com/viptv-org/design/blob/83d338b6ffc1fc5e7f14ad4059f6159b8ee84509/specs/behavior/torrent-native-android.md).

Core validates native transport and returns decisions; platform adapters and the
backend still supply effects, qualification and authorization. This API does not
activate native capability. `PlaybackSession` and `PlaybackLease` remain ordinary
HTTP models and contain no native grant. A qualified Android request may add the
closed `client.native_torrent` extension after successful scoped negotiation.
Omitting it preserves the existing serialized request and direct/gateway shape.

## String bridge operations

Use the existing native UniFFI / browser WASM `normalize(kind, input, origin)`.

| Kind / input | Output |
| --- | --- |
| `playbackProtocolV2` / original successful HTTP response text | `PlaybackProtocol`: `version: 1`, `nativeTorrentVersions: []` or `[1]` |
| `request` / `{"operation":"playbackProtocolV2"}` | `ApiRequest`: GET `/api/v2/playback-protocol`, `body: null` |
| `request` / `{"operation":"playbackV2CancelRequest","requestId":"request_example"}` | `ApiRequest`: DELETE `/api/v2/playback-requests/request_example`, `body: null` |
| `nativeTorrent` / `{"operation":"negotiation", ...NativeTorrentNegotiationFacts}` | `rejectStale`, `authRecovery`, `legacy` or `advertise` |
| `nativeTorrent` / `{"operation":"recovery","facts": NativeTorrentRecoveryFacts}` | typed explicit recovery decision |
| `nativeTorrent` / `{"operation":"releaseResponse","body":"{\"ok\":true}"}` | true; malformed, duplicate or extra fields fail |

`body: null` means no HTTP body, not the four JSON bytes `null`. These are
relative backend control routes; the existing account/profile/device-authorized
adapter supplies authentication. No token/header is serialized in `ApiRequest`.
Cancellation identifiers are 1..128 ASCII letters/digits/`-`/`_`; percent-encoded
aliases, path/query/fragment separators, whitespace and Unicode are rejected.
The added request bridge inputs are closed and reject duplicates, unknown fields
and wrong types. Playback v2 inputs use closed raw validation even when the
native extension is absent; repeated operations cannot bypass dispatch. Valid
legacy request serialization remains unchanged.

The response boundary checks UTF-8 byte length before JSON parsing: maximum
4096 bytes including whitespace. The original text is mandatory. Do not decode
into a JSON object and stringify it before invoking core: that would erase
repeated fields and numeric spellings. Closed serde deserialization rejects
missing, duplicate (including escaped-name duplicates), unknown and wrong-type
fields. Borrowed raw JSON values require exactly the integer token `1`, never
`1.0`, exponent notation, strings, booleans or coercion. The versions array has
zero or one element, only integer `1`. All refusals return the static
`Invalid server response or input`; parser diagnostics/private input are not
exposed. The bridge accepts Rust strings: transport must reject invalid UTF-8
while decoding bytes, never use lossy replacement.

## Adapter and adoption boundary

Core does not perform HTTP or turn protocol support into authority. A consumer
must first enforce HTTPS for the configured backend, existing resource
authorization, `Cache-Control: no-store`, zero redirects, a bounded read and a
5-second total negotiation deadline. Only a valid 200 is parsed as support.
The cancellation extension uses release's 10-second deadline/zero redirects;
only negotiated extension flows may call it. Status/auth recovery, origin and
authenticated-generation fencing, old-server fallback, cancellation effects,
native qualification remain separate integration work. A typed `PlaybackProtocol` is support information, never an admission or
capability decision. TV-web/desktop/Roku must not advertise native support.

## Private grant authority

`NativeTorrentBridge` is a separate nonserializable, redacted native/WASM object.
Construct it with generated `NativeTorrentContext` JSON: configured HTTPS origin,
opaque authenticated scope, generation, qualified/negotiated/VOD facts and the
exact `PlaybackV2Request`. It accepts only qualified Android platforms carrying
the exact version/policy extension. Every instance owns one request/generation.

Pass bounded original identity-encoded HTTP bytes to
`acceptBytes(status, bytes, NativeTorrentObservation JSON)`. The byte entry rejects
malformed UTF-8, never replaces it. `accept` exists for already validated Rust
text. The response parser rejects duplicate/unknown/mixed fields and preserves
integer token spelling before any generic JSON projection. Native metainfo must
be a canonical info-only bencode wrapper; bounded validation checks exact SHA-1,
v1 keys, file/index/size/piece counts, path safety and NFC/case collision/overlap.
Magnet input is exactly the authorized hash with no source-supplied peer hints.

Observations supply scope/generation/sequence, start/poll/heartbeat operation,
suspend-aware receipt/RTT, bounded uncertainty and a trusted wall-clock upper
bound. The core computes receipt plus lifetime minus RTT minus uncertainty minus
one second, checks both wall and monotonic expiry and never extends on poll.
Only successful current-grant heartbeat may renew; grant identity/input remain
immutable. Stale scoped/sequence results cannot replace or revoke current
authority. Invalid current responses, terminal leases and close retire it.

`authorize(NativeTorrentClock JSON)` checks current scope/generation, clocks and
backend revalidation. The engine-only `privateInfoHash`, `privateInputKind`,
`privateInputValue`, `privateFileIndex`, `privateExpectedFileSize` getters require
that same current clock. The generated scalar `metadataMatchesNative(infoHash, fileIndex, fileCount,
selectedFileSize, validatedV1Metadata, clock JSON)` checks
the engine's validated metadata without choosing another file or constructing a
serializable private DTO. `metadataMatches(private facts JSON, clock JSON)` is the
raw private-facts entry. `invalidate()`
latches closed, `playbackId()` supports release, and `state()` returns only safe
transport facts. Private values never enter ordinary JSON state, Debug/toString,
cards, source, playback/history models or diagnostics; ordinary normalizers
reject private transport objects and reflected magnet strings. The adapter must
keep getter results transient and avoid diagnostics/cache backup.

`native_torrent_policy` supplies direct Rust backend admission and request
idempotency decisions using backend-injected resource/session/source proofs.
Private typed grants have no Serialize/Facet implementation; the dedicated
`native_ready_response` serializer is transient HTTP transport only.
`NativeTorrentRecoveryFacts` names explicit Retry/ChooseSource/Back intent.
Retirement must finish before a native retry; only a non-authorization/non-selection
retry of retired native authority returns `forceGatewayRetry`. No automatic
recovery starts gateway delivery.

Regenerate at the canonical source with `cargo run --locked -p viptv-typegen`,
`bash scripts/build-native-bindings.sh` and `bash scripts/build-wasm.sh` using
wasm-bindgen CLI 0.2.92. `tests/playback-protocol-vectors.json` preserves raw text
for native regressions and `node scripts/test-playback-protocol.mjs`; the latter
runs the real generated WASM and a native corpus runner and compares every result
and static error. The ordinary WASM suite includes this parity check.
`tests/native-torrent-vectors.json` and `node scripts/test-native-torrent.mjs`
exercise the actual private holder, original bytes, grant clocks, transitions,
request/recovery decisions and reflected-data privacy through both runtimes.

Adopt one immutable committed core revision in Android and TV-web together using
each consumer's `scripts/core-sync.mjs`; import generated Kotlin/native/WASM from
that same source, check both pins/hashes and run each consumer's local checks.
Do not edit vendored snapshots or activate native capability during adoption.
