# Shared Crux core

Contract: https://github.com/viptv-org/design/blob/c58c9b91827a39442462797602bded35a83a9f64/SHARED_CORE.md

One Rust implementation owns shared parsing and session state for the KMP app and the backend. It supplies actual Crux state transitions, native bridges, generated language interfaces and bounded platform HTTP adapters. App adoption and remaining migration are listed in README.md.

CORE-003 adds Vizio SmartCast controller behavior to the same Rust library for Android. Rust owns pairing, command serialization, remote mappings, current-input/hash sequencing, one stale-setting retry, response interpretation and Conjure launch payloads. The app owns exact-TV-origin HTTPS and the OS credential vault. SmartCast is not exposed to the desktop or web builds.

Acceptance: native execution of the existing application core; generated interface checks; HTTP cancellation/binary/status/origin handling; startup restoration and catalog/playback normalization against backend shapes. SmartCast native contracts cover pairing, every remote mapping, serialized input/setting workflows, retry limits, origin fencing and credential redaction. Hardware and full app migration remain separately qualified.
