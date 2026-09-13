# get-air/vizio integration assessment

Inspected 2026-09-13 at upstream commit `124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d`.

## What the project provides

`get-air/vizio` controls Vizio SmartCast TVs over their local HTTPS API: pairing, remote keys/text, power/audio, input/settings changes, state, app launching, and known-host or caller-supplied `/24` discovery. It is not a media player, codec library, or SDK for running VIPTV on every TV. Conjure launches a reachable hosted HTML URL using app ID `17`, namespace `4`; it does not install an application or add a permanent launcher tile. HTTP power-on needs the TV API reachable (normally Quick Start). Wake-on-LAN and native mDNS/SSDP discovery are absent. [Upstream README](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/README.md)

There is no Rust crate. The repository contains TypeScript Promise/Effect APIs and a Kotlin Multiplatform port. TypeScript declares `@get-air/vizio` version `0.1.0`, depending on `@get-air/http`, `@get-air/cache`, and Effect. Kotlin declares `com.getair:vizio:0.1.0-SNAPSHOT`, Ktor/coroutines/serialization, JVM 17, Android min SDK 24, and targets JVM, Android, Linux x64, Windows x64, macOS x64/arm64, iOS device/simulator, JavaScript, and Wasm. These are configured build targets, not evidence that every target has passed validation. The parity ledger leaves volume/mute/channel commands and release matrices incomplete. GitHub's releases API returned an empty release list; registry publication was not checked. [Package manifest](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/package.json), [Gradle targets](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/build.gradle.kts), [parity ledger](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/PORTING.md), [releases](https://github.com/get-air/vizio/releases).

License: MIT, copyright 2026 Air contributors. A source port must retain the copyright and permission notice in its third-party license notices. [License](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/LICENSE)

## Recommended core seam

Add a portable Rust `smartcast` module to `crates/viptv-core`, exposed through the native JSON/UniFFI bridge. Port protocol behavior rather than embed Kotlin or JavaScript in Rust. Keep request construction, validated host/port and command inputs, pairing/status parsing, remote code tables, input/settings hash transitions, and app/Conjure payloads in Rust. Emit a dedicated SmartCast transport request for an injected platform adapter. Do not reuse the VIPTV backend HTTP transport with globally relaxed TLS or place TV bearer credentials in the ordinary application model/view.

The adapter owns HTTPS execution, exact-origin TLS policy, cancellation/timeouts, response-size limits, and credential-vault access. Serialize requests per television; pairing adopts the returned token immediately. Preserve the same pairing device ID. Inputs must fetch the current-input hash immediately before writing the target input's `CNAME`; settings permit one refetch/retry on `HASHVAL_ERROR`. Handle case-insensitive response fields and protocol status failures even when HTTP returns 200. [Protocol](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/src/Protocol.ts), [client state and commands](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/src/Client.ts), [remote codes](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/src/Remote.ts).

## Selected product scope

The owner clarified that this integration targets Android mobile and Tauri desktop. Browser, Android TV, Tizen, Vizio-hosted, and Roku adapters are excluded from this delivery even where protocol code could theoretically compile.

## Platform feasibility

| Consumer | Portable Rust behavior | Execution requirement / limitation |
| --- | --- | --- |
| Android | Native core + generated Kotlin interface | Dedicated OkHttp client, TV-origin-only TLS exception, Android Keystore; upstream has analogous adapters. Physical SmartCast validation remains required. |
| Tauri desktop | Native core or shared bridge | Native HTTP avoids WebView CORS; TLS exceptions must be scoped to the selected TV, with OS credential storage. Upstream's default TypeScript Tauri profile store writes the bearer token to its store file and should not be copied as the secure-storage design. |
| Browser, Android TV, Tizen, Vizio hosted, Roku | Excluded | No adapter or product support is delivered here. Conjure remains a launch target controlled from Android mobile or Tauri desktop. |

The upstream Android/JVM helper fences relaxed TLS to one HTTPS host and port; reject off-origin requests and redirects in an equivalent native adapter. Public app-catalog traffic must retain normal TLS and a separate transport. Kotlin profile JSON excludes the token structurally and requires a separate vault; supplied vaults cover Android Keystore, Apple Keychain, Windows Credential Manager, and KDE Wallet. Browser/Wasm deliberately has no durable secure vault. [Scoped TLS implementation](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/src/jvmAndAndroidMain/kotlin/com/getair/vizio/SmartCastTls.kt), [platform and storage contract](https://github.com/get-air/vizio/blob/124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d/README.md).

These findings distinguish code portability from network execution and device acceptance. No consumer app, installed device, LAN television, or production service was changed by this assessment.
