//! VIPTV behavior shared by browser WASM and native shells. Roku is independent.
pub mod app;
pub mod domain;
pub mod dto;
pub mod policy;
#[cfg(feature = "provider")]
pub mod provider;
pub mod vizio;
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

/// Plan one SmartCast request without performing network or credential I/O.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn vizio_request(operation: String, input: String) -> String {
    serde_json::to_string(&vizio::plan_request(&operation, &input))
        .expect("SmartCast request results are serializable")
}

/// Interpret SmartCast HTTP and protocol status without exposing transport details.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn vizio_response(status: u16, body: String, allow_statusless: bool) -> String {
    serde_json::to_string(&vizio::parse_response(status, &body, allow_statusless))
        .expect("SmartCast response results are serializable")
}

/// Return the bounded modern/legacy host probe order for one caller-approved /24.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn vizio_discovery_candidates(subnet: String) -> String {
    let result = vizio::discovery_candidates(&subnet);
    serde_json::to_string(&result).expect("SmartCast discovery results are serializable")
}

/// Report whether this target can execute SmartCast directly or needs a LAN bridge.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn vizio_platform_support(platform: String) -> String {
    serde_json::to_string(&vizio::platform_support(&platform))
        .expect("SmartCast support is serializable")
}

/// Stateful SmartCast workflow used by Android mobile and Tauri desktop.
///
/// The shell executes each returned request and resolves it by ID. The bridge
/// keeps pairing credentials, fresh hash values, retry state, and command
/// serialization out of UI code.
#[cfg_attr(feature = "native", derive(uniffi::Object))]
pub struct SmartCastBridge {
    inner: std::sync::Mutex<vizio::VizioController>,
}

#[cfg_attr(feature = "native", uniffi::export)]
impl SmartCastBridge {
    #[cfg_attr(feature = "native", uniffi::constructor)]
    pub fn new(config: String) -> Result<Self, CoreError> {
        Ok(Self {
            inner: std::sync::Mutex::new(
                vizio::VizioController::new(&config).map_err(|_| CoreError::InvalidInput)?,
            ),
        })
    }

    pub fn start(&self, operation: String, input: String) -> Result<String, CoreError> {
        let output = self
            .inner
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .start(&operation, &input);
        serialize_smartcast(output)
    }

    pub fn resolve(&self, request_id: u32, status: u16, body: String) -> Result<String, CoreError> {
        let output = self
            .inner
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .resolve(request_id, status, &body);
        serialize_smartcast(output)
    }

    pub fn reject(&self, request_id: u32) -> Result<String, CoreError> {
        let output = self
            .inner
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .reject(request_id);
        serialize_smartcast(output)
    }

    pub fn cancel(&self) -> Result<(), CoreError> {
        self.inner.lock().map_err(|_| CoreError::Bridge)?.cancel();
        Ok(())
    }

    /// Return the in-memory pairing credential only to the native vault adapter.
    pub fn credential(&self) -> Result<Option<String>, CoreError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .auth_token())
    }

    pub fn clear_credential(&self) -> Result<(), CoreError> {
        self.inner
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .clear_auth_token();
        Ok(())
    }
}

fn serialize_smartcast(output: vizio::VizioControllerOutput) -> Result<String, CoreError> {
    serde_json::to_string(&output).map_err(|_| CoreError::Bridge)
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
        "cardPresentation" => check::<dto::CardPresentation>(v),
        "catalog" => check::<dto::Catalog>(v),
        "catalogs" => check::<Vec<dto::Catalog>>(v),
        "media" => check::<dto::MediaItem>(v),
        "source" => check::<dto::MediaSource>(v),
        "playback" => check::<dto::PlaybackSession>(v),
        "discover" => check::<dto::DiscoverPage>(v),
        _ => Ok(()),
    }
}
