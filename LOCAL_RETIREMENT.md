# BE-002 anonymous application bridge retirement

Baseline: `fba95c8f3ba00e97fbc460acc746d23a912795bc`. The optional application
`provider` feature, its eight provider/addon exports and Tauri forwarding feature
are removed. Generated native Kotlin and actual WASM are rebuilt by the provided
scripts, not edited manually. The standalone provider crate remains because the
backend vendors it (`backend/scripts/sync-provider.sh`); it has no tests here, and
its behaviour is covered by the backend's suite. The four removed provider-bridge
unit tests and old WASM bridge block tested the retired wrappers, not
authenticated v2 playback. Domain/Stremio normalizers, Crux session/events/effects,
runtime transports and SmartCast are unchanged.

`node scripts/test-v2-retirement-parity.mjs BASELINE_CORE_CHECKOUT` compares 35
v2/request/response/startup operations across baseline/candidate native and actual
WASM, checks byte-identical generated JSON protocol/type declarations and absence
of retired WASM/native-Kotlin exports. Use an unmodified baseline source/artifact
checkout at the revision above; it compiles the native example but never modifies
source or account data. Ordinary Core/WASM/runtime tests remain separate gates.

TV-web adoption is coordinated next. Android's isolated category work and user
UI checkout, desktop pins and production are deliberately not changed here.
No physical device or installed application qualification follows from bindings.
