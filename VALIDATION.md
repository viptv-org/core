# First shared-core checkpoint — 2026-09-13

- Crux 0.20.0 native library and browser WASM compile.
- 14 Rust contracts pass, including storage/session restoration, token refresh retaining profile identity, malformed/unsupported catalog isolation, source sanitization and media capability URL checks.
- `node scripts/test-wasm.mjs` passes three startup vectors also executed by the native Rust tests, plus domain boundary checks through the actual generated WASM artifact.
- Formatting and strict workspace/all-target Clippy pass. Native shared library and Kotlin UniFFI bindings generated successfully.
- Nine TypeScript runtime/transport tests pass: correlated effect dispatch, native Tauri command mapping, binary/status/header preservation, URL policy, redirects, cancellation, deadlines and body limits. The Tauri plugin is injected in these tests; they are not an installed desktop qualification.
- Kotlin HTTP adapter supports OkHttp PATCH and has Android instrumentation test source. No Gradle, emulator, physical Android or installed Tauri validation was run in this checkpoint.

TV-web adopts the Rust normalizers and generated domain types. Its own browser/playback evidence belongs in tv-web/TESTING.md. Existing Android/desktop applications have not yet adopted the core. Existing TV navigation/session orchestration still has remaining extraction work; this is a working first library release, not a completed all-platform migration.

Strict TypeScript validation also passed against the actual pinned Tauri HTTP plugin types; the runtime imports Rust-generated JSON wire types directly.
