//! VIPTV behavior shared by browser WASM and native shells. Roku is independent.
pub mod app;
pub mod domain;
pub mod dto;
pub mod policy;
pub use app::*;
use crux_core::{
    Core,
    bridge::{Bridge, EffectId, JsonFfiFormat},
};

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "native", derive(uniffi::Error))]
pub enum CoreError {
    #[error("Invalid server response or input")]
    InvalidInput,
    #[error("Could not process core request")]
    Bridge,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "native", derive(uniffi::Object))]
pub struct CoreBridge {
    inner: Bridge<Viptv, JsonFfiFormat>,
}
impl Default for CoreBridge {
    fn default() -> Self {
        Self {
            inner: Bridge::new(Core::new()),
        }
    }
}

#[cfg_attr(feature = "native", uniffi::export)]
impl CoreBridge {
    #[cfg_attr(feature = "native", uniffi::constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&self, event: String) -> Result<String, CoreError> {
        let mut output = vec![];
        self.inner
            .update(event.as_bytes(), &mut output)
            .map_err(|_| CoreError::Bridge)?;
        String::from_utf8(output).map_err(|_| CoreError::Bridge)
    }
    pub fn resolve(&self, id: u32, result: String) -> Result<String, CoreError> {
        let mut output = vec![];
        self.inner
            .resolve(EffectId(id), result.as_bytes(), &mut output)
            .map_err(|_| CoreError::Bridge)?;
        String::from_utf8(output).map_err(|_| CoreError::Bridge)
    }
    pub fn view(&self) -> Result<String, CoreError> {
        let mut output = vec![];
        self.inner
            .view(&mut output)
            .map_err(|_| CoreError::Bridge)?;
        String::from_utf8(output).map_err(|_| CoreError::Bridge)
    }
}
#[cfg_attr(feature = "native", uniffi::export)]
pub fn normalize(kind: String, input: String, origin: String) -> Result<String, CoreError> {
    if input.len() > 2 * 1024 * 1024 {
        return Err(CoreError::InvalidInput);
    }
    let value = serde_json::from_str(&input).map_err(|_| CoreError::InvalidInput)?;
    let normalized = domain::normalize_value(&kind, &value, &origin)?;
    validate_normalized(&kind, &normalized)?;
    serde_json::to_string(&normalized).map_err(|_| CoreError::InvalidInput)
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl CoreBridge {
    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        Self::new()
    }
    #[wasm_bindgen::prelude::wasm_bindgen(js_name=update)]
    pub fn wasm_update(&self, event: String) -> Result<String, wasm_bindgen::JsValue> {
        self.update(event).map_err(|e| e.to_string().into())
    }
    #[wasm_bindgen::prelude::wasm_bindgen(js_name=resolve)]
    pub fn wasm_resolve(&self, id: u32, result: String) -> Result<String, wasm_bindgen::JsValue> {
        self.resolve(id, result).map_err(|e| e.to_string().into())
    }
    #[wasm_bindgen::prelude::wasm_bindgen(js_name=view)]
    pub fn wasm_view(&self) -> Result<String, wasm_bindgen::JsValue> {
        self.view().map_err(|e| e.to_string().into())
    }
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name=normalize)]
pub fn wasm_normalize(
    kind: String,
    input: String,
    origin: String,
) -> Result<String, wasm_bindgen::JsValue> {
    normalize(kind, input, origin).map_err(|e| e.to_string().into())
}
#[cfg(feature = "native")]
uniffi::setup_scaffolding!();

fn validate_normalized(kind: &str, v: &serde_json::Value) -> Result<(), CoreError> {
    fn check<T: serde::de::DeserializeOwned>(v: &serde_json::Value) -> Result<(), CoreError> {
        serde_json::from_value::<T>(v.clone())
            .map(|_| ())
            .map_err(|_| CoreError::InvalidInput)
    }
    match kind {
        "catalog" => check::<dto::Catalog>(v),
        "catalogs" => check::<Vec<dto::Catalog>>(v),
        "media" => check::<dto::MediaItem>(v),
        "source" => check::<dto::MediaSource>(v),
        "playback" => check::<dto::PlaybackSession>(v),
        "discover" => check::<dto::DiscoverPage>(v),
        _ => Ok(()),
    }
}
