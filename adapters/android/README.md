# Android transport

Include `src/main/kotlin` in the native Android shell source set. This adapter uses Android's built-in `org.json` and OkHttp. Pin `implementation("com.squareup.okhttp3:okhttp:5.5.0")` in the consuming module; 5.5.0 was verified from Maven Central's published metadata/POM on 2026-09-13. Declare `android.permission.INTERNET`. Supply a bounded background executor and exact authorized backend origins. The shell owns that executor and shuts it down when appropriate.

For each `{id,effect:{Http:request}}` returned by the native Rust bridge, call `execute(JSONObject(requestJson))`, then pass the callback's JSON to `core.resolve(id, result.toString())` on the core owner's thread. Forward resulting effects through the same dispatcher. Retain the returned `Call` and cancel it on teardown or scope changes. The Rust core also discards stale results by epoch. Do not instantiate WASM on Android.

Responses preserve binary bytes as unsigned JSON numbers, duplicate header values, and all HTTP statuses (including 4xx/5xx) as `Ok`. Redirects are disabled before networking; URL credentials and origins outside the configured set are rejected. Cancellation, size-limit failures and sanitized IO errors resolve once as `Err.Io`; socket timeouts resolve as `Err.Timeout`. OkHttp call timeout bounds the complete exchange, in addition to connect/read limits. Automatic retries are disabled so mutation retry policy stays in the core. Android's network security policy controls any explicitly authorized cleartext development origin.

PATCH and empty POST/PUT/PATCH bodies are supported. This adapter has not yet been compiled in an Android application or qualified on hardware. The main Android app remains unchanged.
