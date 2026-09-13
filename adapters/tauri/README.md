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
