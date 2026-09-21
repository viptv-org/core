//! SmartCast host adapter for the Tauri desktop shell.
//! SmartCast traffic stays native; the React renderer never sees TV credentials or TLS policy.

use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use url::Url;
use viptv_core::SmartCastBridge;

const CREDENTIAL_SERVICE: &str = "org.viptv.smartcast";

pub struct SmartCastState {
    session: Mutex<Option<Session>>,
    cancelled: AtomicBool,
    cancel_notify: Notify,
}

impl Default for SmartCastState {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
            cancelled: AtomicBool::new(false),
            cancel_notify: Notify::new(),
        }
    }
}

struct Session {
    bridge: SmartCastBridge,
    client: reqwest::Client,
    origin: String,
    credential_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlannedRequest {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<Map<String, Value>>,
    timeout_millis: u64,
    max_response_bytes: u64,
}

#[tauri::command]
pub async fn smartcast_configure(
    state: tauri::State<'_, SmartCastState>,
    host: String,
    device_id: String,
    device_name: String,
    credential_id: String,
) -> Result<(), String> {
    if credential_id.is_empty()
        || credential_id.len() > 128
        || !credential_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err("Invalid SmartCast credential identifier".into());
    }
    let origin = configured_origin(&host)?;
    let credential = keyring::Entry::new(CREDENTIAL_SERVICE, &credential_id)
        .map_err(|_| "SmartCast credential vault is unavailable")?
        .get_password()
        .ok();
    let bridge = SmartCastBridge::new(
        json!({
            "host": origin,
            "authToken": credential,
            "deviceId": device_id,
            "deviceName": device_name,
        })
        .to_string(),
    )
    .map_err(|_| "Invalid SmartCast configuration")?;
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "SmartCast transport is unavailable")?;
    *state.session.lock().await = Some(Session {
        bridge,
        client,
        origin,
        credential_id,
    });
    Ok(())
}

#[tauri::command]
pub async fn smartcast_run(
    state: tauri::State<'_, SmartCastState>,
    operation: String,
    input: String,
) -> Result<String, String> {
    state.cancelled.store(false, Ordering::Release);
    let mut guard = state.session.lock().await;
    let session = guard
        .as_mut()
        .ok_or_else(|| "SmartCast is not configured".to_owned())?;
    let mut output = session
        .bridge
        .start(operation, input)
        .map_err(|_| "SmartCast core is unavailable")?;
    loop {
        let parsed: Value =
            serde_json::from_str(&output).map_err(|_| "SmartCast core returned invalid output")?;
        if parsed.get("kind").and_then(Value::as_str) != Some("request") {
            if parsed
                .get("credentialChanged")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                let token = session
                    .bridge
                    .credential()
                    .map_err(|_| "SmartCast core is unavailable")?
                    .ok_or_else(|| "SmartCast pairing did not return a credential".to_owned())?;
                keyring::Entry::new(CREDENTIAL_SERVICE, &session.credential_id)
                    .map_err(|_| "SmartCast credential vault is unavailable")?
                    .set_password(&token)
                    .map_err(|_| "Could not store SmartCast credential")?;
            }
            return Ok(output);
        }
        let request_id = parsed
            .get("requestId")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| "SmartCast core returned an invalid request ID".to_owned())?;
        let request = serde_json::from_value::<PlannedRequest>(
            parsed.get("request").cloned().unwrap_or(Value::Null),
        )
        .map_err(|_| "SmartCast core returned an invalid request")?;
        output = match execute(session, request, &state.cancelled, &state.cancel_notify).await {
            Ok((status, body)) => session.bridge.resolve(request_id, status, body),
            Err(()) => session.bridge.reject(request_id),
        }
        .map_err(|_| "SmartCast core is unavailable")?;
    }
}

#[tauri::command]
pub fn smartcast_cancel(state: tauri::State<'_, SmartCastState>) {
    state.cancelled.store(true, Ordering::Release);
    state.cancel_notify.notify_one();
}

#[tauri::command]
pub async fn smartcast_forget(state: tauri::State<'_, SmartCastState>) -> Result<(), String> {
    state.cancelled.store(true, Ordering::Release);
    state.cancel_notify.notify_one();
    let mut guard = state.session.lock().await;
    if let Some(session) = guard.as_mut() {
        session
            .bridge
            .clear_credential()
            .map_err(|_| "SmartCast core is unavailable")?;
        match keyring::Entry::new(CREDENTIAL_SERVICE, &session.credential_id)
            .map_err(|_| "SmartCast credential vault is unavailable")?
            .delete_credential()
        {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err("Could not clear SmartCast credential".into()),
        }
    }
    Ok(())
}

async fn execute(
    session: &Session,
    request: PlannedRequest,
    cancelled: &AtomicBool,
    cancel_notify: &Notify,
) -> Result<(u16, String), ()> {
    if cancelled.load(Ordering::Acquire)
        || request.timeout_millis < 250
        || request.timeout_millis > 60_000
        || request.max_response_bytes == 0
        || request.max_response_bytes > 8 * 1024 * 1024
        || request_origin(&request.url).ok().as_deref() != Some(session.origin.as_str())
    {
        return Err(());
    }
    let method = reqwest::Method::from_bytes(request.method.as_bytes()).map_err(|_| ())?;
    let mut builder = session
        .client
        .request(method, &request.url)
        .timeout(Duration::from_millis(request.timeout_millis));
    for (name, value) in request.headers {
        builder = builder.header(name, value);
    }
    if let Some(body) = request.body {
        builder = builder.json(&body);
    }
    let response = tokio::select! {
        response = builder.send() => response.map_err(|_| ())?,
        _ = wait_for_cancel(cancelled, cancel_notify) => return Err(()),
    };
    if response.status().is_redirection()
        || response
            .content_length()
            .is_some_and(|length| length > request.max_response_bytes)
    {
        return Err(());
    }
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(());
        }
        let chunk = tokio::select! {
            chunk = stream.next() => chunk,
            _ = wait_for_cancel(cancelled, cancel_notify) => return Err(()),
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk.map_err(|_| ())?;
        if bytes.len().saturating_add(chunk.len()) > request.max_response_bytes as usize {
            return Err(());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes)
        .map(|body| (status, body))
        .map_err(|_| ())
}

async fn wait_for_cancel(cancelled: &AtomicBool, notify: &Notify) {
    while !cancelled.load(Ordering::Acquire) {
        notify.notified().await;
    }
}

fn configured_origin(input: &str) -> Result<String, String> {
    let url = parse_https(input)?;
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err("Invalid SmartCast TV origin".into());
    }
    let authority = input
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(input)
        .split('/')
        .next()
        .unwrap_or_default();
    let explicit_port = if authority.starts_with('[') {
        authority
            .find(']')
            .is_some_and(|end| authority.as_bytes().get(end + 1) == Some(&b':'))
    } else {
        authority.rsplit_once(':').is_some_and(|(_, port)| {
            !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
        })
    };
    origin_string(
        &url,
        explicit_port
            .then(|| url.port_or_known_default())
            .flatten()
            .unwrap_or(7345),
    )
}

fn request_origin(input: &str) -> Result<String, String> {
    let url = parse_https(input)?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Invalid SmartCast TV origin".into());
    }
    origin_string(&url, url.port_or_known_default().unwrap_or(443))
}

fn parse_https(input: &str) -> Result<Url, String> {
    let url = if input.contains("://") {
        Url::parse(input)
    } else {
        Url::parse(&format!("https://{input}"))
    }
    .map_err(|_| "Invalid SmartCast TV origin")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.host_str().is_none()
    {
        return Err("Invalid SmartCast TV origin".into());
    }
    Ok(url)
}

fn origin_string(url: &Url, port: u16) -> Result<String, String> {
    let host = url.host_str().ok_or("Invalid SmartCast TV origin")?;
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_owned()
    };
    Ok(format!("https://{host}:{port}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_defaults_to_smartcast_port_and_preserves_explicit_port() {
        assert_eq!(
            configured_origin("192.0.2.10").unwrap(),
            "https://192.0.2.10:7345"
        );
        assert_eq!(
            configured_origin("https://example.test:443").unwrap(),
            "https://example.test:443"
        );
    }

    #[test]
    fn request_origin_ignores_path_but_rejects_credentials_and_queries() {
        assert_eq!(
            request_origin("https://192.0.2.10:7345/key_command/").unwrap(),
            "https://192.0.2.10:7345"
        );
        assert!(request_origin("https://token@192.0.2.10:7345/key").is_err());
        assert!(request_origin("https://192.0.2.10:7345/key?next=other").is_err());
    }
}
