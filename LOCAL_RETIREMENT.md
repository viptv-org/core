# BE-002 anonymous application bridge retirement

Baseline: `fba95c8f3ba00e97fbc460acc746d23a912795bc`. The optional application
`provider` feature, its eight provider/addon exports and Tauri forwarding feature
are removed. Generated native Kotlin is rebuilt by the provided
scripts, not edited manually. The standalone provider crate remains because the
backend vendors it (`backend/scripts/sync-provider.sh`); it has no tests here, and
its behaviour is covered by the backend's suite. The four removed provider-bridge
unit tests and old WASM bridge block tested the retired wrappers, not
authenticated v2 playback. Domain/Stremio normalizers, Crux session/events/effects,
runtime transports and SmartCast are unchanged.

TV-web adoption is coordinated next. Android's isolated category work and user
UI checkout, desktop pins and production are deliberately not changed here.
No physical device or installed application qualification follows from bindings.
