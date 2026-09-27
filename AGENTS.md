# viptv shared core

Delivery policy (owner approved 2026-09-27): only Android, desktop, Roku and TV-web
build workflows remain, triggered by main pushes and manual dispatch. No PR
gates, automatic releases, image publishing or deployment. Retain local checks.
This supersedes older automation/release-gate instructions below.

Read DESIGN_REF and design/SHARED_CORE.md before behavior changes. The design repository owns UI/UX; core owns shared application state, data normalization and decisions. Keep transport/storage/player work in platform adapters and business rules in Rust. Generate Kotlin/TypeScript interfaces from Rust; never hand-edit generated bindings.

Coordinate with consumers before changing events/effects. Preserve account/profile/source identity, cancellation, protected-profile checks and direct-first playback. Every release pins core, generated bindings and adapters together. Record which apps actually adopt it; a build is not hardware acceptance.

Use repository GitHub Issues for specs and tasks. Finish the code changes before the final test batch as requested by the owner. Build/type generation is part of implementation. Never publish credentials, real tokens, screenshots or provider URLs.
