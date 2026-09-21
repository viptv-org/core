//! Native Tauri integration source for the shared VIPTV core.
//!
//! Hosts depend on this crate by path and register the commands they need:
//! `commands::{core_update, core_resolve, core_view}` for the native core
//! bridge and `smartcast::{SmartCastState, smartcast_configure, smartcast_run,
//! smartcast_cancel, smartcast_forget}` for TV pairing. SmartCast traffic
//! stays native; renderers never see TV credentials or TLS policy.
//!
//! This is integration source, not a standalone packaged Tauri app. Browser,
//! Android TV, Tizen, Vizio-hosted, and Roku adapters live elsewhere.
pub mod commands;
pub mod smartcast;
