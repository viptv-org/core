//! Pure provider/addon client helpers bridged out to fat shells.
//!
//! Compiled behind the `provider` feature, which the web build turns on for
//! local-mode bundles. Each helper converts JSON strings and returns JSON
//! strings; no credential, network, or storage logic runs here, so shells
//! can plan discovery, rank candidates, and build playable URLs without
//! reimplementing normalization rules.
use serde_json::Value;

/// Bridge failure carrying the shared provider message. UniFFI cannot throw
/// bare strings, so native shells catch this and read the detail; the wasm
/// twin throws the same text.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "native", derive(uniffi::Error))]
pub enum ProviderBridgeError {
    // `detail` avoids colliding with Kotlin's Throwable.message.
    #[error("{detail}")]
    Provider { detail: String },
}

impl ProviderBridgeError {
    fn message(message: impl Into<String>) -> Self {
        Self::Provider {
            detail: message.into(),
        }
    }
}

type BridgeResult = Result<String, ProviderBridgeError>;

fn fail(message: impl Into<String>) -> ProviderBridgeError {
    ProviderBridgeError::message(message)
}

const MAX_BRIDGE_INPUT: usize = 4 * 1024 * 1024;

fn parse(input: &str) -> Result<Value, String> {
    if input.len() > MAX_BRIDGE_INPUT {
        return Err("Input too large".to_string());
    }
    serde_json::from_str(input).map_err(|_| "Invalid input".to_string())
}

fn typed<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, String> {
    if input.len() > MAX_BRIDGE_INPUT {
        return Err("Input too large".to_string());
    }
    serde_json::from_str(input).map_err(|_| "Invalid input".to_string())
}

fn out<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("bridge output is always serializable")
}

/// Build a relative endpoint path for one addon manifest URL.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn addon_endpoint(base: String, parts: String) -> BridgeResult {
    if base.len() > MAX_BRIDGE_INPUT {
        return Err(fail("Input too large"));
    }
    let values = parse(&parts).map_err(fail)?;
    let parts: Vec<&str> = values
        .as_array()
        .ok_or_else(|| fail("Invalid endpoint parts"))?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    viptv_provider::discover::addon_endpoint(&base, &parts).map_err(fail)
}

/// Expose a catalog's extra options (genres and the like) as wire objects.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn addon_catalog_extras(catalog: String) -> String {
    let Ok(value) = parse(&catalog) else {
        return "[]".to_string();
    };
    let extras = viptv_provider::extras::catalog_extras(&value);
    out(&extras.iter().map(|extra| extra.wire()).collect::<Vec<_>>())
}

/// Whether one addon manifest supports a resource for a kind/id pair.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn addon_supports(manifest: String, resource: String, kind: String, id: String) -> bool {
    match parse(&manifest) {
        Ok(value) => viptv_provider::extras::supports(&value, &resource, &kind, &id),
        Err(_) => false,
    }
}

/// Plan one discovery request for a set of addon entries without network I/O.
///
/// `entries` is a JSON array of `[id, manifest_url, manifest]` tuples. The
/// returned plan describes the endpoints to call and the aggregation flags.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn discover_plan(entries: String, request: String) -> BridgeResult {
    let entries: Vec<(i64, String, Value)> = typed(&entries).map_err(fail)?;
    let request: viptv_provider::discover::DiscoveryRequest = typed(&request).map_err(fail)?;
    let plan = viptv_provider::discover::plan_discovery(&entries, &request).map_err(fail)?;
    Ok(out(&plan))
}

/// Aggregate responses for one discovery plan into a page.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn discover_aggregate(responses: String, plan: String, skip: u64) -> BridgeResult {
    let responses: Vec<Value> = typed(&responses).map_err(fail)?;
    let plan: viptv_provider::discover::DiscoveryPlan = typed(&plan).map_err(fail)?;
    let page = viptv_provider::discover::aggregate_discovery(&responses, &plan, skip as usize)
        .map_err(fail)?;
    Ok(page.to_string())
}

/// Parse one provider row into a candidate, or `null` when it is not playable.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn provider_candidate(provider_id: i64, kind: String, row: String) -> String {
    let Ok(value) = parse(&row) else {
        return "null".to_string();
    };
    match viptv_provider::candidate::candidate_from_json(provider_id, &kind, &value) {
        Some(candidate) => out(&candidate),
        None => "null".to_string(),
    }
}

/// Rank a set of candidates for one match request, returning the winners.
///
/// `candidates` is the JSON array produced by [`provider_candidate`].
#[cfg_attr(feature = "native", uniffi::export)]
pub fn provider_select_candidates(
    kind: String,
    request: String,
    candidates: String,
) -> BridgeResult {
    let request =
        viptv_provider::candidate::MatchRequest::parse(&parse(&request).map_err(fail)?, &kind)
            .map_err(fail)?;
    let candidates: Vec<viptv_provider::candidate::Candidate> = typed(&candidates).map_err(fail)?;
    let selected = viptv_provider::candidate::select_candidates(&candidates, &request);
    let selected: Vec<_> = selected.into_iter().cloned().collect();
    Ok(out(&selected))
}

/// Build a playable Xtream URL for a provider's kind/id/extension.
///
/// `provider` is `{"url": ..., "username": ..., "password": ...}`. The result
/// is returned only to the calling shell for playback on this device.
#[cfg_attr(feature = "native", uniffi::export)]
pub fn provider_media_url(provider: String, kind: String, id: String, ext: String) -> BridgeResult {
    let value = parse(&provider).map_err(|_| fail("Invalid provider"))?;
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| fail("Invalid provider URL"))?;
    let username = value.get("username").and_then(Value::as_str).unwrap_or("");
    let password = value.get("password").and_then(Value::as_str).unwrap_or("");
    viptv_provider::normalize::media_url(url, username, password, &kind, &id, &ext).map_err(fail)
}

/// Browser-side twins of the native exports, so local-mode web bundles call
/// the same provider logic the backend runs.
#[cfg(target_arch = "wasm32")]
mod wasm_export {
    use super::ProviderBridgeError;

    fn error(message: ProviderBridgeError) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&message.to_string())
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=addonEndpoint)]
    pub fn addon_endpoint(base: String, parts: String) -> Result<String, wasm_bindgen::JsValue> {
        super::addon_endpoint(base, parts).map_err(error)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=addonCatalogExtras)]
    pub fn addon_catalog_extras(catalog: String) -> String {
        super::addon_catalog_extras(catalog)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=addonSupports)]
    pub fn addon_supports(manifest: String, resource: String, kind: String, id: String) -> bool {
        super::addon_supports(manifest, resource, kind, id)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=discoverPlan)]
    pub fn discover_plan(
        entries: String,
        request: String,
    ) -> Result<String, wasm_bindgen::JsValue> {
        super::discover_plan(entries, request).map_err(error)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=discoverAggregate)]
    pub fn discover_aggregate(
        responses: String,
        plan: String,
        skip: u64,
    ) -> Result<String, wasm_bindgen::JsValue> {
        super::discover_aggregate(responses, plan, skip).map_err(error)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=providerCandidate)]
    pub fn provider_candidate(provider_id: i64, kind: String, row: String) -> String {
        super::provider_candidate(provider_id, kind, row)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=providerSelectCandidates)]
    pub fn provider_select_candidates(
        kind: String,
        request: String,
        candidates: String,
    ) -> Result<String, wasm_bindgen::JsValue> {
        super::provider_select_candidates(kind, request, candidates).map_err(error)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=providerMediaUrl)]
    pub fn provider_media_url(
        provider: String,
        kind: String,
        id: String,
        ext: String,
    ) -> Result<String, wasm_bindgen::JsValue> {
        super::provider_media_url(provider, kind, id, ext).map_err(error)
    }
}

#[cfg(all(test, feature = "provider"))]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{"id":"express","catalogs":[{"type":"movie","id":"top","name":"Top Movies","extra":[{"name":"genre","isRequired":false,"options":["Action","Drama"]}]}],"resources":["catalog","meta","stream"],"types":["movie"]}"#;

    #[test]
    fn plans_and_aggregates_discovery() {
        let entries = format!("[[1,\"https://example.com/manifest.json\",{MANIFEST}]]");
        let request = r#"{"kind":"movie","catalog":"top","skip":0,"extras":{}}"#;
        let plan: viptv_provider::discover::DiscoveryPlan =
            serde_json::from_str(&discover_plan(entries, request.to_string()).expect("plan"))
                .expect("plan json");
        assert_eq!(plan.endpoints.len(), 1);
        assert!(plan.endpoints[0].starts_with("https://example.com/catalog/movie/top.json"));
        let responses = r#"[{"metas":[{"type":"movie","id":"m1","name":"Alpha","poster":null}]}]"#;
        let page: serde_json::Value = serde_json::from_str(
            &discover_aggregate(
                responses.to_string(),
                serde_json::to_string(&plan).unwrap(),
                0,
            )
            .expect("aggregate"),
        )
        .expect("page json");
        if page["metas"].as_array().map(Vec::len) != Some(1) {
            panic!("unexpected page: {page}");
        }
        assert!(!page["has_more"].as_bool().unwrap());
        assert!(page["next_skip"].is_null());
    }

    #[test]
    fn selects_candidates_by_ids_and_title() {
        let row = r#"{"num":1,"name":"Test Movie (2010)","stream_type":"movie","stream_id":101,"container_extension":"mp4"}"#;
        let candidate = provider_candidate(7, "movie".into(), row.into());
        let parsed: serde_json::Value = serde_json::from_str(&candidate).unwrap();
        assert_eq!(parsed["id"], "iptv:7:movie:101");
        let ballot = format!("[{candidate}]");
        let request = r#"{"id":"tt0123456","name":"Test Movie","year":2010,"imdb_id":"tt0123456"}"#;
        let selected =
            provider_select_candidates("movie".into(), request.into(), ballot).expect("select");
        let picked: Vec<serde_json::Value> = serde_json::from_str(&selected).unwrap();
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0]["id"], "iptv:7:movie:101");
    }

    #[test]
    fn builds_playable_urls_and_rejects_bad_providers() {
        let provider = r#"{"url":"https://example.com","username":"demo","password":"secret"}"#;
        let url = provider_media_url(provider.into(), "movie".into(), "101".into(), "mp4".into())
            .expect("url");
        assert!(url.ends_with("/movie/demo/secret/101.mp4"), "url: {url}");
        assert!(provider_media_url("{}".into(), "movie".into(), "1".into(), "mp4".into()).is_err());
    }

    #[test]
    fn exposes_catalog_extras_and_support() {
        let catalog = r#"{"type":"movie","id":"top","extra":[{"name":"genre","isRequired":false,"options":["Action","Drama"]}]}"#;
        let extras: Vec<serde_json::Value> =
            serde_json::from_str(&addon_catalog_extras(catalog.into())).unwrap();
        assert_eq!(extras[0]["name"], "genre");
        assert!(addon_supports(
            MANIFEST.into(),
            "catalog".into(),
            "movie".into(),
            "top".into()
        ));
        assert!(!addon_supports(
            MANIFEST.into(),
            "catalog".into(),
            "series".into(),
            "top".into()
        ));
    }
}
