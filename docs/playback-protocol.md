# Playback protocol foundation

Normative contract: design `83d338b6ffc1fc5e7f14ad4059f6159b8ee84509`,
[SRC-TORRENT-NATIVE-001](https://github.com/viptv-org/design/blob/83d338b6ffc1fc5e7f14ad4059f6159b8ee84509/specs/behavior/torrent-native-android.md).

This is an inactive parsing/request foundation, not native playback admission,
qualification, grant handling or capability activation. Existing direct/HLS
normalization, `PlaybackSession`, `PlaybackLease`, `PlaybackClient` and start
request shapes remain unchanged. `client.native_torrent` is not accepted or sent.

## String bridge operations

Use the existing native UniFFI / browser WASM `normalize(kind, input, origin)`.

| Kind / input | Output |
| --- | --- |
| `playbackProtocolV2` / original successful HTTP response text | `PlaybackProtocol`: `version: 1`, `nativeTorrentVersions: []` or `[1]` |
| `request` / `{"operation":"playbackProtocolV2"}` | `ApiRequest`: GET `/api/v2/playback-protocol`, `body: null` |
| `request` / `{"operation":"playbackV2CancelRequest","requestId":"request_example"}` | `ApiRequest`: DELETE `/api/v2/playback-requests/request_example`, `body: null` |

`body: null` means no HTTP body, not the four JSON bytes `null`. These are
relative backend control routes; the existing account/profile/device-authorized
adapter supplies authentication. No token/header is serialized in `ApiRequest`.
Cancellation identifiers are 1..128 ASCII letters/digits/`-`/`_`; percent-encoded
aliases, path/query/fragment separators, whitespace and Unicode are rejected.
The added request bridge inputs are closed and reject duplicates, unknown fields
and wrong types without changing legacy request-input compatibility.

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
response release validation and native qualification remain separate integration
work. A typed `PlaybackProtocol` is support information, never an admission or
capability decision. TV-web/desktop/Roku must not advertise native support.

Regenerate at the canonical source with `cargo run --locked -p viptv-typegen`,
`bash scripts/build-native-bindings.sh` and `bash scripts/build-wasm.sh` using
wasm-bindgen CLI 0.2.92. `tests/playback-protocol-vectors.json` preserves raw text
for native regressions and `node scripts/test-playback-protocol.mjs`; the latter
runs the real generated WASM and a native corpus runner and compares every result
and static error. The ordinary WASM suite includes this parity check.

Adopt one immutable committed core revision in Android and TV-web together using
each consumer's `scripts/core-sync.mjs`; import generated Kotlin/native/WASM from
that same source, check both pins/hashes and run each consumer's local checks.
Do not edit vendored snapshots or activate native capability during adoption.
