//! Backend wire normalization. Unknown/add-on data never gains transport authority.
use crate::CoreError;
use serde_json::{Map, Value, json};
type Result<T> = std::result::Result<T, CoreError>;
fn timestamp(v: &Value) -> Option<f64> {
    v.as_f64()
        .map(|n| if n < 1e12 { n * 1000.0 } else { n })
        .or_else(|| {
            v.as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.timestamp_millis() as f64)
        })
}
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
        .filter(|s| matches!(*s, "movie" | "series" | "live" | "episode"))
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
    for (a, b) in [
        ("primary", "primary"),
        ("is_primary", "primary"),
        ("avatar_style", "avatarStyle"),
        ("avatar_choice", "avatarChoice"),
    ] {
        copy(v, &mut out, a, b);
    }
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
    let catalog_type = v["type"].as_str().unwrap_or("movie");
    if catalog_type.trim().is_empty() || catalog_type.len() > 64 {
        return Err(invalid());
    }
    let mut out = json!({"id":id(v,"id")?,"name":fallback(v,&["name","id"],"Catalog"),"type":catalog_type,"supportsSearch":v["supports_search"].as_bool().unwrap_or(false),"supportsSkip":v["supports_skip"].as_bool().unwrap_or(false),"genres":strings(v,"genres"),"extras":[],"raw":clean(v)});
    optional_string(v, &mut out, "addon_name", "addonName");
    optional_number(v, &mut out, "addon_id", "addonId");
    if let Ok(key) = id(v, "addon_id") {
        out["addonKey"] = json!(key);
    }
    out["extras"] = Value::Array(v["extra"].as_array().into_iter().flatten().filter_map(|e| {
        let name = e["name"].as_str().filter(|s|!s.is_empty())?;
        let mut extra = json!({"name":name,"required":e["is_required"].as_bool().unwrap_or(false),"options":strings(e,"options")});
        optional_string(e,&mut extra,"default","defaultValue"); optional_number(e,&mut extra,"options_limit","optionsLimit"); Some(extra)
    }).collect());
    Ok(out)
}
pub fn media(v: &Value) -> Result<Value> {
    let mut raw = clean(v);
    if let Some(fields) = raw.as_object_mut() {
        // These potentially large fields already have typed representations.
        // Retaining them in raw duplicates entire series catalogs and synopses
        // each time a normalized DTO crosses the bounded native/WASM bridge.
        for key in ["videos", "episodes", "description", "overview"] {
            fields.remove(key);
        }
    }
    let mut out = json!({"id":id(v,"id")?,"type":kind(&v["type"])?,"name":fallback(v,&["name","title"],"Untitled"),"title":fallback(v,&["title","name"],"Untitled"),"genres":strings(v,"genres"),"raw":raw});
    // Generic provider logos describe title artwork for on-demand media, but
    // live-channel logos are station identity and must not replace title text.
    let logo_keys = [
        "titleLogo",
        "title_logo",
        "clearLogo",
        "clearlogo",
        "clear_logo",
    ];
    let title_logo = logo_keys
        .iter()
        .find_map(|key| v[*key].as_str().filter(|s| !s.trim().is_empty()))
        .or_else(|| {
            (v["type"] != "live")
                .then(|| v["logo"].as_str().filter(|s| !s.trim().is_empty()))
                .flatten()
        });
    if let Some(logo) = title_logo {
        out["titleLogo"] = json!(logo);
    }
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
    for (keys, to) in [
        (vec!["thumbnail", "landscape", "image"], "thumbnail"),
        (vec!["imdbRating", "imdb_rating", "rating"], "imdbRating"),
        (vec!["posterShape", "poster_shape"], "posterShape"),
        (vec!["credits"], "credits"),
    ] {
        if let Some(value) = keys.iter().find_map(|k| {
            v[*k]
                .as_str()
                .map(str::to_owned)
                .or_else(|| v[*k].as_f64().map(|n| n.to_string()))
        }) {
            out[to] = json!(value);
        }
    }
    if out["credits"].is_null() {
        let mut credits = vec![];
        for (key, prefix) in [("director", "Directed by "), ("cast", "Starring ")] {
            let names = v[key].as_str().map(str::to_owned).unwrap_or_else(|| {
                strings(v, key)
                    .into_iter()
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(", ")
            });
            if !names.is_empty() {
                credits.push(format!("{prefix}{names}"));
            }
        }
        if !credits.is_empty() {
            out["credits"] = json!(credits.join("\n"));
        }
    }
    for (keys, to) in [
        (vec!["updated_at", "updatedAt"], "updatedAtMillis"),
        (
            vec!["released", "released_at", "releaseDate"],
            "releasedAtMillis",
        ),
    ] {
        if let Some(n) = keys.iter().find_map(|k| timestamp(&v[*k])) {
            out[to] = json!(n);
        }
    }
    if let Some(year) = v["year"]
        .as_str()
        .or_else(|| v["releaseInfo"].as_str())
        .and_then(|s| s.get(..4))
        .and_then(|s| s.parse::<f64>().ok())
    {
        out["year"] = json!(year);
    }
    out["episodes"] = json!([]);
    if let Some(episodes) = v["videos"].as_array().or_else(|| v["episodes"].as_array()) {
        out["episodes"] = Value::Array(
            episodes
                .iter()
                .filter_map(|ep| {
                    let mut ep = ep.clone();
                    if ep["type"].is_null() {
                        ep["type"] = v["type"].clone();
                    }
                    if ep["series_id"].is_null() {
                        ep["series_id"] = v["id"].clone();
                    }
                    let mut item = media(&ep).ok()?;
                    if item["episodeTitle"].is_null() && item["name"] != out["name"] {
                        item["episodeTitle"] = item["name"].clone();
                    }
                    item["name"] = out["name"].clone();
                    if item["titleLogo"].is_null() && !out["titleLogo"].is_null() {
                        item["titleLogo"] = out["titleLogo"].clone();
                    }
                    Some(item)
                })
                .collect(),
        );
    }
    for key in ["year", "position", "duration", "season", "episode"] {
        optional_number(v, &mut out, key, key);
    }
    if v["watch_state"] == "watched" {
        out["watched"] = json!(true);
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
        json!({"headers":v["headers"].as_object().map(|h|h.iter().filter(|(_,v)|v.is_string()).map(|(k,v)|(k.clone(),v.clone())).collect::<Map<String,Value>>()).unwrap_or_default(),"id":string(v,"id")?,"url":url.as_str(),"format":fallback(v,&["format"],"hls"),"mode":fallback(v,&["mode"],"direct"),"videoMode":fallback(v,&["video_mode"],"copy"),"audioMode":fallback(v,&["audio_mode"],"copy"),"position":v["position"].as_f64().unwrap_or(0.0),"live":v["live"].as_bool().unwrap_or(false),"duration":v["duration"].as_f64().unwrap_or(0.0),"audioTracks":v["audio_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitleTracks":v["subtitle_tracks"].as_array().into_iter().flatten().filter(|v|v.is_object()).map(track).collect::<Result<Vec<_>>>()?,"subtitlesSupported":v["subtitles_supported"].as_bool().unwrap_or(false)}),
    )
}
pub fn normalize_value(kind_name: &str, v: &Value, origin: &str) -> Result<Value> {
    match kind_name {
        "pairing" => Ok(
            json!({"deviceCode":string(v,"device_code")?,"userCode":string(v,"user_code")?,"verificationUri":string(v,"verification_uri")?,"verificationUriComplete":string(v,"verification_uri_complete")?,"qrUri":string(v,"qr_uri")?,"expiresIn":number(v,"expires_in")?,"intervalSeconds":number(v,"interval")?}),
        ),
        "androidPreferences" => {
            let mut input = json!({"audio_language":"en","subtitle_language":"en","subtitles_enabled":false,"subtitle_size":"normal","subtitle_style":"system","quality":"auto","autoplay":true});
            if let Some(fields) = v.as_object() {
                for (k, value) in fields {
                    if !value.is_null() {
                        input[k] = value.clone();
                    }
                }
            }
            normalize_value("preferences", &input, origin)
        }
        "preferences" => {
            for (key, allowed) in [
                ("subtitle_size", vec!["small", "normal", "large"]),
                ("subtitle_style", vec!["system", "shadow", "opaque"]),
                ("quality", vec!["auto", "1080p", "720p", "480p"]),
            ] {
                if !allowed.contains(&string(v, key)?.as_str()) {
                    return Err(invalid());
                }
            }
            let mut out = json!({});
            for (a, b) in [
                ("audio_language", "audioLanguage"),
                ("subtitle_language", "subtitleLanguage"),
                ("subtitle_size", "subtitleSize"),
                ("subtitle_style", "subtitleStyle"),
                ("quality", "quality"),
            ] {
                out[b] = json!(string(v, a)?);
            }
            for (a, b) in [
                ("subtitles_enabled", "subtitlesEnabled"),
                ("autoplay", "autoplay"),
            ] {
                out[b] = json!(boolean(v, a)?);
            }
            Ok(out)
        }
        "page" => Ok(
            json!({"items":array(v,"items")?.iter().filter_map(|m|media(m).ok()).collect::<Vec<_>>(),"offset":number(v,"offset")?,"total":number(v,"total")?,"nextOffset":v["next_offset"]}),
        ),
        "discoverResponse" => {
            let response = &v["response"];
            let items = array(response, "metas")?
                .iter()
                .filter_map(|m| {
                    let mut m = m.clone();
                    if m["type"].is_null() {
                        m["type"] = v["type"].clone();
                    }
                    media(&m).ok()
                })
                .collect::<Vec<_>>();
            let mut out =
                json!({"items":items,"hasMore":response["has_more"].as_bool().unwrap_or(false)});
            optional_number(response, &mut out, "next_skip", "nextSkip");
            Ok(out)
        }
        "detailResponse" => {
            let mut raw = v["response"]["meta"].clone();
            obj(&raw)?;
            if raw["type"].is_null() {
                raw["type"] = v["item"]["type"].clone();
            }
            let item = media(&raw)?;
            Ok(json!({"episodes":item["episodes"],"item":item}))
        }
        "streamPoll" => {
            let events=array(v,"events")?.iter().map(|e|{let mut out=json!({"sequence":number(e,"seq")?,"source":string(e,"source")?,"sources":array(e,"streams")?.iter().filter_map(|s|source(s).ok()).collect::<Vec<_>>()});if e["error"].is_string(){out["error"]=json!("Source unavailable");}Ok(out)}).collect::<Result<Vec<_>>>()?;
            Ok(json!({"events":events,"done":boolean(v,"done")?}))
        }
        "live" => {
            let channels = v["channels"]
                .as_array()
                .or_else(|| v["items"].as_array())
                .ok_or_else(invalid)?
                .iter()
                .filter_map(|m| {
                    let mut m = m.clone();
                    if m["type"].is_null() {
                        m["type"] = json!("live");
                    }
                    if m["poster"].is_null() {
                        m["poster"] = m["logo"].clone();
                    }
                    let category = m["section"]
                        .as_str()
                        .or_else(|| m["category"].as_str())
                        .map(str::to_owned);
                    let mut item = media(&m).ok()?;
                    if let Some(category) = category {
                        item["category"] = json!(category);
                    }
                    Some(item)
                })
                .collect::<Vec<_>>();
            Ok(
                json!({"total":v["total"].as_u64().unwrap_or(channels.len() as u64),"channels":channels,"searchScope":v["search_scope"]}),
            )
        }
        "liveCategories" => Ok(
            json!({"categories":array(v,"categories")?.iter().map(|c|Ok(json!({"id":id(c,"id")?,"name":string(c,"name")?,"count":number(c,"count")?,"raw":clean(c)}))).collect::<Result<Vec<_>>>()?,"total":v["total"].as_f64().unwrap_or(0.0)}),
        ),
        "guide" => {
            let programs = v["programs"]
                .as_array()
                .or_else(|| v["programmes"].as_array())
                .or_else(|| v["items"].as_array())
                .ok_or_else(invalid)?;
            Ok(
                json!({"timezone":fallback(v,&["timezone"],""),"timeline":v["timeline"].as_array().into_iter().flatten().map(|t|Ok(json!({"time":number(t,"start")?,"displayTime":fallback(t,&["display_time"],"")}))).collect::<Result<Vec<_>>>()?,"programs":programs.iter().map(|p|{let mut out=json!({"title":fallback(p,&["title"],"No schedule available"),"start":p["start"].as_f64().or_else(||p["start_time"].as_f64()).unwrap_or(0.0),"end":p["end"].as_f64().or_else(||p["end_time"].as_f64()).unwrap_or(0.0),"displayTime":fallback(p,&["display_time"],""),"raw":clean(p)});optional_string(p,&mut out,"description","description");out}).collect::<Vec<_>>()}),
            )
        }
        "profile" => profile(v),
        "identity" => identity(v),
        "tokens" => tokens(v),
        "catalog" => catalog(v),
        "media" => media(v),
        "source" => source(v),
        "mediaArray" => Ok(Value::Array(
            v.as_array()
                .ok_or_else(invalid)?
                .iter()
                .filter_map(|m| media(m).ok())
                .collect(),
        )),
        "preferencesRequest" => crate::policy::normalize(kind_name, v),
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
            let mut out = json!({"items":v["metas"].as_array().or_else(||v["items"].as_array()).or_else(||v["rows"].as_array()).ok_or_else(invalid)?.iter().filter_map(|m|media(m).ok()).collect::<Vec<_>>(),"hasMore":v["has_more"].as_bool().unwrap_or(false)});
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
        _ => crate::policy::normalize(kind_name, v),
    }
}
