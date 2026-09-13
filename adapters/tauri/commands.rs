//! Copy into the consuming Tauri host; links the real native core, never WASM.
use std::sync::Mutex;
use viptv_core::CoreBridge;

// SmartCast commands live in smartcast.rs because they use a dedicated native
// HTTPS client and OS credential vault. Do not route TV requests through the
// backend HTTP plugin or expose them to the React renderer.

pub struct CoreState(Mutex<CoreBridge>);

impl Default for CoreState {
    fn default() -> Self {
        Self(Mutex::new(CoreBridge::new()))
    }
}

#[tauri::command]
pub fn core_update(state: tauri::State<'_, CoreState>, event: String) -> Result<String, String> {
    state
        .0
        .lock()
        .map_err(|_| "Core unavailable".to_owned())?
        .update(event)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn core_resolve(
    state: tauri::State<'_, CoreState>,
    id: u32,
    result: String,
) -> Result<String, String> {
    state
        .0
        .lock()
        .map_err(|_| "Core unavailable".to_owned())?
        .resolve(id, result)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn core_view(state: tauri::State<'_, CoreState>) -> Result<String, String> {
    state
        .0
        .lock()
        .map_err(|_| "Core unavailable".to_owned())?
        .view()
        .map_err(|e| e.to_string())
}
