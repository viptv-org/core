# viptv shared core

Read DESIGN_REF and design/SHARED_CORE.md before behavior changes. The design repository owns UI/UX; core owns shared application state, data normalization and decisions. Keep transport/storage/player work in platform adapters and business rules in Rust. Generate Kotlin/TypeScript interfaces from Rust; never hand-edit generated bindings.

Coordinate with consumers before changing events/effects. Preserve account/profile/source identity, cancellation, protected-profile checks and direct-first playback. Every release pins core, generated bindings and adapters together. Record which apps actually adopt it; a build is not hardware acceptance.

Use repository GitHub Issues for specs and tasks. Finish the code changes before the final test batch as requested by the owner. Build/type generation is part of implementation. Keep emulators and local Gradle off. Bound browser/Node workers and compilation memory. Never publish credentials, real tokens, screenshots or provider URLs.
