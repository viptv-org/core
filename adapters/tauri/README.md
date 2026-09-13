# Native Tauri core bridge

`commands.rs` contains the actual command implementations for `viptv_core::CoreBridge`. Copy it into the consuming host's source tree as `mod commands;`, depend on this repository's `crates/viptv-core`, and register:

```rust
tauri::Builder::default()
    .manage(commands::CoreState::default())
    .plugin(tauri_plugin_http::init())
    .invoke_handler(tauri::generate_handler![
        commands::core_update, commands::core_resolve, commands::core_view,
    ])
    // Continue with the application's existing setup and run context.
```

The renderer invokes `core_update` with `{event: JSON.stringify(event)}`, `core_resolve` with `{id, result: JSON.stringify(result)}`, and `core_view` with no arguments. Each returns JSON text. Send every returned HTTP effect to `createTauriHttpTransport` and every Storage effect to the native secure storage adapter. Read the view on Render. The mutex serializes access to one native Rust instance; the renderer never instantiates WASM. Register these commands for the trusted local application window only, and scope the HTTP plugin to exact backend URLs.

This is integration source, not a standalone packaged Tauri app. Compile it as part of host adoption; current existing applications remain unchanged.

## SmartCast desktop controller

Copy `smartcast.rs` beside `commands.rs` and register `SmartCastState` plus `smartcast_configure`, `smartcast_run`, `smartcast_cancel`, and `smartcast_forget`. The host needs `reqwest` with JSON and stream support, `futures-util`, `keyring`, `tokio` sync, `url`, `serde`, and `serde_json`. Restrict these commands to the trusted application window.

`smartcast_run` keeps the full Rust request/resolve workflow in one native command. React submits an operation and JSON input, then receives only the final complete/error object. The dedicated reqwest client rejects redirects and off-origin effects before networking; its invalid-certificate allowance is confined to requests generated for the configured television origin. Pairing credentials use the operating system keyring and never enter the Tauri store, frontend state, command errors, or logs. The normal backend and public-network clients retain normal TLS validation.

This adapter is for the Tauri desktop application. Browser, Android TV, Tizen, Vizio-hosted, and Roku SmartCast adapters are outside this delivery. No installed desktop or physical-TV qualification is implied by the source adapter.
