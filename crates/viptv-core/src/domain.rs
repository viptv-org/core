//! Backend wire normalization. Unknown/add-on data never gains transport authority.
use crate::CoreError;
use serde_json::{Map, Value, json};
type Result<T> = std::result::Result<T, CoreError>;
fn invalid() -> CoreError {
    CoreError::InvalidInput
}
fn obj(v: &Value) -> Result<&Map<String, Value>> {
    v.as_object().ok_or_else(invalid)
}
fn string(v: &Value, key: &str) -> Result<String> {
    v[key]
        .as_str()
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .ok_or_else(invalid)
}
fn id(v: &Value, key: &str) -> Result<String> {
    if let Some(s) = v[key].as_str().filter(|s| !s.is_empty()) {
        return Ok(s.into());
    }
    if let Some(n) = v[key]
        .as_i64()
        .filter(|n| n.unsigned_abs() <= 9_007_199_254_740_991)
    {
        return Ok(n.to_string());
    }
    Err(invalid())
}
fn boolean(v: &Value, key: &str) -> Result<bool> {
    v[key].as_bool().ok_or_else(invalid)
}
fn number(v: &Value, key: &str) -> Result<f64> {
    v[key].as_f64().ok_or_else(invalid)
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v[key].as_array().ok_or_else(invalid)
}
fn strings(v: &Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn copy(v: &Value, out: &mut Value, from: &str, to: &str) {
    if !v[from].is_null() {
        out[to] = v[from].clone();
    }
}
fn optional_string(v: &Value, out: &mut Value, from: &str, to: &str) {
    if v[from].is_string() {
        out[to] = v[from].clone();
    }
}
fn optional_number(v: &Value, out: &mut Value, from: &str, to: &str) {
    if v[from].is_number() {
        out[to] = v[from].clone();
    }
}
fn kind(v: &Value) -> Result<&str> {
    v.as_str()
        .filter(|s| matches!(*s, "movie" | "series" | "live"))
        .ok_or_else(invalid)
}
fn fallback(v: &Value, keys: &[&str], default: &str) -> String {
    keys.iter()
        .find_map(|k| v[*k].as_str())
        .unwrap_or(default)
        .into()
}
pub fn clean(v: &Value) -> Value {
    match v {
        Value::Array(items) => Value::Array(items.iter().map(clean).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(k, _)| {
                    let k = k.to_lowercase().replace(['_', '-'], "");
                    ![
                        "url",
                        "uri",
                        "link",
                        "header",
                        "authorization",
                        "accesstoken",
                        "refreshtoken",
                        "devicecode",
                        "devicetoken",
                        "cookie",
                        "password",
                        "credential",
                        "proxy",
                        "referer",
                        "origin",
                    ]
                    .iter()
                    .any(|s| k.contains(s))
                })
                .map(|(k, v)| (k.clone(), clean(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}
pub fn profile(v: &Value) -> Result<Value> {
    obj(v)?;
    let mut out = json!({"id":id(v,"id")?,"name":string(v,"name")?,"raw":clean(v)});
    if let Some(avatar) = v["avatar"].as_str().or_else(|| v["avatar_url"].as_str()) {
        out["avatar"] = json!(avatar);
    }
    if let Some(kid) = v["kids"]
        .as_bool()
        .or_else(|| v["kid"].as_bool())
        .or_else(|| v["is_kids"].as_bool())
    {
        out["kid"] = json!(kid);
    }
    if let Some(complete) = v["setup_complete"].as_bool() {
        out["setupComplete"] = json!(complete);
    }
    Ok(out)
}
pub fn identity(v: &Value) -> Result<Value> {
    let a = &v["account"];
    Ok(
        json!({"account":{"id":id(a,"id")?,"username":string(a,"username")?,"name":string(a,"name")?,"role":string(a,"role")?},
        "profiles":array(v,"profiles")?.iter().map(profile).collect::<Result<Vec<_>>>()?,
        "profileId":id(v,"profile_id").ok(),"restricted":boolean(v,"restricted")?,"profileSetupRequired":boolean(v,"profile_setup_required")?}),
    )
}
pub fn tokens(v: &Value) -> Result<Value> {
    Ok(
        json!({"sessionId":id(v,"session_id")?,"accountId":id(v,"account_id")?,"profileId":id(v,"profile_id").ok(),"accessToken":string(v,"access_token")?,"refreshToken":string(v,"refresh_token")?,"expiresIn":number(v,"expires_in")?}),
    )
}
pub fn catalog(v: &Value) -> Result<Value> {
    let mut out = json!({"id":id(v,"id")?,"name":fallback(v,&["name","id"],"Catalog"),"type":kind(v.get("type").unwrap_or(&json!("movie")))?,"supportsSearch":v["supports_search"].as_bool().unwrap_or(false),"supportsSkip":v["supports_skip"].as_bool().unwrap_or(false),"genres":strings(v,"genres"),"extras":[],"raw":clean(v)});
    optional_number(v, &mut out, "addon_id", "addonId");
    out["extras"] = Value::Array(v["extra"].as_array().into_iter().flatten().filter_map(|e| {
        let name = e["name"].as_str().filter(|s|!s.is_empty())?;
        let mut extra = json!({"name":name,"required":e["is_required"].as_bool().unwrap_or(false),"options":strings(e,"options")});
        optional_string(e,&mut extra,"default","defaultValue"); optional_number(e,&mut extra,"options_limit","optionsLimit"); Some(extra)
    }).collect());
    Ok(out)
}
pub fn media(v: &Value) -> Result<Value> {
    let mut out = json!({"id":id(v,"id")?,"type":kind(&v["type"])?,"name":fallback(v,&["name","title"],"Untitled"),"title":fallback(v,&["title","name"],"Untitled"),"genres":strings(v,"genres"),"raw":clean(v)});
    for (from, to) in [
        ("poster", "poster"),
        ("runtime", "runtime"),
        ("series_id", "seriesId"),
        ("queue_status", "queueStatus"),
        ("source_addon_id", "sourceAddonId"),
        ("source_name", "sourceName"),
        ("source_fingerprint", "sourceFingerprint"),
        ("source_binge_group", "sourceBingeGroup"),
        ("source_release_group", "sourceReleaseGroup"),
        ("source_quality", "sourceQuality"),
        ("source_audio", "sourceAudio"),
    ] {
        optional_string(v, &mut out, from, to);
    }
    for (keys, to) in [
        (["background", "backdrop"], "background"),
        (["description", "overview"], "description"),
        (["episodeTitle", "episode_title"], "episodeTitle"),
    ] {
        if let Some(s) = keys.iter().find_map(|k| v[*k].as_str()) {
            out[to] = json!(s);
        }
    }
    for key in ["year", "position", "duration", "season", "episode"] {
        optional_number(v, &mut out, key, key);
    }
    if v["watched"].is_boolean() {
        copy(v, &mut out, "watched", "watched");
    }
    if v["previous_episode"].is_object() {
        out["previousEpisode"] = media(&v["previous_episode"])?;
    }
    Ok(out)
}
fn source(v: &Value) -> Result<Value> {
    let mut out =
        json!({"id":string(v,"id")?,"name":fallback(v,&["name","title"],"Source"),"raw":clean(v)});
    for (a, b) in [
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
    let url = base.join(&string(v, "url")?).map_err(|_| invalid())?;
    if !matches!(base.scheme(), "https" | "http")
        || url.origin() != base.origin()
        || !url.path().starts_with("/media/")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid());
    }
    Ok(
        json!({"id":string(v,"id")?,"url":url.as_str(),"format":fallback(v,&["format"],"hls"),"mode":fallback(v,&["mode"],"direct"),"videoMode":fallback(v,&["video_mode"],"copy"),"audioMode":fallback(v,&["audio_mode"],"copy"),"position":v["position"].as_f64().unwrap_or(0.0),"live":v["live"].as_bool().unwrap_or(false),"duration":v["duration"].as_f64().unwrap_or(0.0),"audioTracks":v["audio_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitleTracks":v["subtitle_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitlesSupported":v["subtitles_supported"].as_bool().unwrap_or(false)}),
    )
}
pub fn normalize_value(kind_name: &str, v: &Value, origin: &str) -> Result<Value> {
    match kind_name {
        "profile" => profile(v),
        "identity" => identity(v),
        "tokens" => tokens(v),
        "catalog" => catalog(v),
        "media" => media(v),
        "source" => source(v),
        "playback" => playback(v, origin),
        "clean" => Ok(clean(v)),
        "catalogs" => Ok(Value::Array(
            v.as_array()
                .ok_or_else(invalid)?
                .iter()
                .filter_map(|v| catalog(v).ok())
                .collect(),
        )),
        "discover" => {
            let mut out = json!({"items":array(v,"metas")?.iter().map(media).collect::<Result<Vec<_>>>()?,"hasMore":v["has_more"].as_bool().unwrap_or(false)});
            optional_number(v, &mut out, "next_skip", "nextSkip");
            Ok(out)
        }
        "container" => {
            let url = url::Url::parse(&string(v, "url")?).map_err(|_| invalid())?;
            let path = url.path().to_lowercase();
            Ok(json!(if path.ends_with(".m3u8") {
                "hls"
            } else if path.ends_with(".mpd") {
                "dash"
            } else if path.ends_with(".mp4") || path.ends_with(".m4v") {
                "mp4"
            } else {
                "unknown"
            }))
        }
        _ => Err(invalid()),
    }
}
