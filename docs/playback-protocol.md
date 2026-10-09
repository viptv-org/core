# Playback protocol foundation

## Shared runtime v2 integration foundation

Design `16277caf2c7bf16296e25da40f143666465f0d2b` authorizes client integration
with measured performance work deferred. `torrentRuntimeProtocol` parses the
separate closed version-2 response and requests GET
`/api/v2/torrent-runtime-protocol` with no body. Native v1 parsing and its route
remain strict and separate. Both native Rust and actual WASM reject mixed
versions, duplicate fields and non-integer version tokens.

`torrent_runtime::TorrentRuntimeGrant` is a private, redacted transport value
with tracker hints and optional file/archive selection. Its dedicated response
serializer and transition validator retain immutable identity and sixty-second
authority. Runtime v2 request capability is allowed only for Android, Android TV
and desktop. The native v1 authority object refuses that capability, preventing
a v2 request from silently adopting v1 authority.

This checkpoint supplies schema and negotiation foundations. The v2 private
client authority object, backend admission/renewal adoption and platform
effects are still integration work; it does not activate an installed client.

## Native v1 contract

Normative contract: design `5b68802c6f5ea91dc248defb047439f3ea96cce5`,
[SRC-TORRENT-NATIVE-001](https://github.com/viptv-org/design/blob/5b68802c6f5ea91dc248defb047439f3ea96cce5/specs/behavior/torrent-native-android.md).

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
| `nativeTorrent` / `{"operation":"failure","reason":"native_payload_limit"}` | canonical user-safe `{code, message}` for a closed observed failure fact |
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

The failure projection accepts at most 512 UTF-8 bytes of original JSON, with
only `operation` and a known `reason`; duplicate, extra and wrong-type fields
fail with the same static input error. It accepts no exception text, URL, hash
or path. Native failure codes also use the same canonical copy through
`apiError` and failed `playbackV2` leases. The adapter reports the observed cause:
metadata timeout and total acquisition timeout remain distinct, and neither
asserts that peers are absent. Storage/cache and aggregate 2 GiB payload refusal
require their corresponding observations. Unknown exceptions become the closed
`native_playback_failed` reason. Presentation does not change recovery decisions.

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


## Shared streaming transport v2

`TorrentRuntimeBridge` is the private v2 holder, negotiated separately at
`/api/v2/torrent-runtime-protocol`. It preserves tracker hints, nullable file and
archive indexes and the original source descriptor. V1 parsing and behavior are
unchanged. Its byte-only adoption, clock checks, renewal and redacted state use
native and actual-WASM bindings from one revision. `bindResolution` records the
runtime's verified file/member and media length; explicit selections and repeated
resolution must agree. Renewal cannot replace that identity.

`normalize("torrentRuntime", ...)` supplies closed v2 negotiation, retirement-aware
recovery, factual stage labels and stage-specific failures. Retired native retry
returns `ordinaryRetry`, obtaining fresh native authority. It never requests a
gateway fallback. Cached bytes remain independent of these transient grants.
The Android and desktop adapters own worker IO, supervision and player effects.
