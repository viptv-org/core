use serde_json::{Value, json};

/// Display facts only. Never promote arbitrary response content into UI diagnostics.
pub fn api_error(v: &Value) -> Value {
    let status = v["status"].as_u64().unwrap_or(0);
    let raw = v["error"].as_str().unwrap_or("").trim();
    let supplied = v["error_code"].as_str().unwrap_or("");
    let code = match (supplied, raw) {
        (
            "",
            "Provider connection limit reached"
            | "All available connections are busy. Try this channel again shortly.",
        ) => "provider_connection_limit",
        ("", "Playback capacity reached") => "playback_capacity",
        ("", "Stream expired; discover again") => "source_expired",
        _ => supplied,
    };
    let known = match code {
        "provider_connection_limit" => Some(
            "This IPTV provider has reached its connection limit. Stop another stream or choose another provider.",
        ),
        "playback_capacity" => Some(
            "The server has reached its playback limit. Stop another stream or try again later.",
        ),
        "source_expired" => {
            Some("This stream has expired. Refresh the sources and choose it again.")
        }
        "source_access_denied" => Some(
            "The provider rejected access to this stream. Check the provider account or choose another source.",
        ),
        "source_unavailable" => {
            Some("The provider could not be reached. Try again or choose another source.")
        }
        "delivery_unsupported" => Some(
            "This source cannot be played with the current playback configuration. Choose another source.",
        ),
        "parent_required" => Some("Parent PIN required"),
        "parent_pin_invalid" => Some("Incorrect parent PIN"),
        "profile_required" => Some("Profile selection required"),
        "profile_policy_changed" => Some("Profile policy changed"),
        _ => None,
    };
    let lower = raw.to_ascii_lowercase();
    let protocol = matches!(
        raw,
        "authorization_pending" | "slow_down" | "expired_token" | "access_denied" | "invalid_grant"
    );
    let safe = protocol
        || !raw.is_empty()
            && raw.chars().count() <= 240
            && !raw.chars().any(char::is_control)
            && ![
                "://",
                "bearer ",
                "authorization",
                "cookie",
                "password",
                "token=",
                "token:",
                "secret=",
                "<",
                ">",
                "traceback",
                "stack trace",
            ]
            .iter()
            .any(|part| lower.contains(part));
    let fallback = match status {
        0 => "Could not reach the server. Check your connection and try again.",
        401 => "Your session has expired. Sign in or pair this device again.",
        403 => "VIPTV refused this request. Check your profile permissions.",
        404 | 410 => "This item or stream is no longer available. Refresh and try again.",
        429 => "Too many requests. Wait a moment and try again.",
        502 | 503 | 504 => {
            "The server or provider is temporarily unavailable. Try again or choose another source."
        }
        _ => "VIPTV could not complete this request. Try again.",
    };
    json!({"message": known.unwrap_or(if safe {raw} else {fallback}), "code": code})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_is_not_rate_limiting_and_legacy_servers_work() {
        let legacy = api_error(&json!({"status":429,"error":"Provider connection limit reached"}));
        let coded = api_error(&json!({"status":429,"error_code":"provider_connection_limit"}));
        assert_eq!(legacy, coded);
        assert!(
            coded["message"]
                .as_str()
                .unwrap()
                .contains("Stop another stream")
        );
        assert!(
            api_error(&json!({"status":429}))["message"]
                .as_str()
                .unwrap()
                .contains("Too many requests")
        );
    }
    #[test]
    fn unknown_errors_are_useful_without_leaking_transport_details() {
        assert_eq!(
            api_error(&json!({"status":400,"error":"authorization_pending"}))["message"],
            "authorization_pending"
        );
        assert_eq!(
            api_error(&json!({"status":400,"error":"No enabled providers"}))["message"],
            "No enabled providers"
        );
        for secret in [
            "https://provider.test/password",
            "Authorization: Bearer secret",
            "cookie=secret",
            "token=secret",
            "<html>gateway</html>",
            "failure\nprivate details",
        ] {
            let result = api_error(&json!({"status":502,"error":secret}));
            assert!(!result["message"].as_str().unwrap().contains(secret));
        }
    }
}
