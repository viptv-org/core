use super::*;

pub(super) fn source(v: &Value) -> Result<Value> {
    let mut out =
        json!({"id":string(v,"id")?,"name":fallback(v,&["name","title"],"Source"),"raw":clean(v)});
    for (a, b) in [
        ("provider", "provider"),
        ("description", "description"),
        ("source_fingerprint", "sourceFingerprint"),
        ("title", "title"),
        ("filename", "filename"),
        ("source_addon_id", "sourceAddonId"),
        ("source_name", "sourceName"),
        ("source_audio", "audio"),
    ] {
        optional_string(v, &mut out, a, b);
    }
    if let Some(q) = v["source_quality"]
        .as_str()
        .or_else(|| v["quality"].as_str())
    {
        out["quality"] = json!(q);
    }
    Ok(out)
}
fn track(v: &Value) -> Result<Value> {
    let mut out = json!({"inputIndex":number(v,"input_index")?,"languageStatus":fallback(v,&["language_status"],"unknown"),"title":fallback(v,&["title"],""),"selected":v["selected"].as_bool().unwrap_or(false),"supported":v["supported"].as_bool().unwrap_or(false),"selectable":v["selectable"].as_bool().unwrap_or(false)});
    optional_string(v, &mut out, "codec", "codec");
    optional_string(v, &mut out, "language", "language");
    Ok(out)
}
pub fn playback(v: &Value, origin: &str) -> Result<Value> {
    let base = url::Url::parse(origin).map_err(|_| invalid())?;
    // Proxy sessions serve root-relative /media/ capability URLs on the API
    // origin. A direct-URL session hands the ORIGINAL absolute source URL to a
    // client that declared direct_urls, so an absolute http(s) URL without
    // embedded credentials passes through unchanged.
    let raw = string(v, "url")?;
    let url = if raw.starts_with('/') {
        let joined = base.join(&raw).map_err(|_| invalid())?;
        if !matches!(base.scheme(), "https" | "http")
            || joined.origin() != base.origin()
            || !joined.path().starts_with("/media/")
            || !joined.username().is_empty()
            || joined.password().is_some()
        {
            return Err(invalid());
        }
        joined
    } else {
        let parsed = url::Url::parse(&raw).map_err(|_| invalid())?;
        if !matches!(parsed.scheme(), "https" | "http")
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return Err(invalid());
        }
        parsed
    };
    let mut out = json!({"headers":v["headers"].as_object().map(|h|h.iter().filter(|(_,v)|v.is_string()).map(|(k,v)|(k.clone(),v.clone())).collect::<Map<String,Value>>()).unwrap_or_default(),"id":string(v,"id")?,"url":url.as_str(),"format":fallback(v,&["format"],"hls"),"mode":fallback(v,&["mode"],"direct"),"videoMode":fallback(v,&["video_mode"],"copy"),"audioMode":fallback(v,&["audio_mode"],"copy"),"position":v["position"].as_f64().unwrap_or(0.0),"live":v["live"].as_bool().unwrap_or(false),"duration":v["duration"].as_f64().unwrap_or(0.0),"audioTracks":v["audio_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitleTracks":v["subtitle_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitlesSupported":v["subtitles_supported"].as_bool().unwrap_or(false)});
    // Source authorization is present only when the session carries one;
    // absent fields are omitted, matching the generated wire types.
    if let Some(authorization) = authorization(v) {
        out["authorization"] = authorization;
    }
    Ok(out)
}

/// Source credentials for a direct-URL session: the server hands over the
/// provider's Cookie/User-Agent so a native engine fetches the original
/// stream itself. Proxy sessions carry none; absent fields are omitted.
fn authorization(v: &Value) -> Option<Value> {
    let source = v["authorization"].as_object()?;
    let mut out = Map::new();
    if let Some(cookie) = source.get("cookie").and_then(Value::as_str) {
        out.insert("cookie".into(), json!(cookie));
    }
    if let Some(user_agent) = source.get("user_agent").and_then(Value::as_str) {
        out.insert("userAgent".into(), json!(user_agent));
    }
    Some(Value::Object(out))
}
