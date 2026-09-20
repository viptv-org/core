use super::playback::source;
use super::*;

pub(super) fn pairing(v: &Value) -> Result<Value> {
    Ok(
        json!({"deviceCode":string(v,"device_code")?,"userCode":string(v,"user_code")?,"verificationUri":string(v,"verification_uri")?,"verificationUriComplete":string(v,"verification_uri_complete")?,"qrUri":string(v,"qr_uri")?,"expiresIn":number(v,"expires_in")?,"intervalSeconds":number(v,"interval")?}),
    )
}

pub(super) fn android_preferences(v: &Value, origin: &str) -> Result<Value> {
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

pub(super) fn preferences(v: &Value) -> Result<Value> {
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

pub(super) fn page(v: &Value) -> Result<Value> {
    Ok(
        json!({"items":array(v,"items")?.iter().filter_map(|m|media(m).ok()).collect::<Vec<_>>(),"offset":number(v,"offset")?,"total":number(v,"total")?,"nextOffset":v["next_offset"]}),
    )
}

pub(super) fn discover_response(v: &Value) -> Result<Value> {
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
    let unsupported_count = array(response, "metas")?
        .iter()
        .filter(|m| {
            let media_type = if m["type"].is_null() {
                &v["type"]
            } else {
                &m["type"]
            };
            m.is_object() && media_type.is_string() && kind(media_type).is_err()
        })
        .count();
    let mut out = json!({"items":items,"hasMore":response["has_more"].as_bool().unwrap_or(false)});
    if unsupported_count > 0 {
        out["unsupportedCount"] = json!(unsupported_count);
    }
    optional_number(response, &mut out, "next_skip", "nextSkip");
    Ok(out)
}

pub(super) fn detail_response(v: &Value) -> Result<Value> {
    let mut raw = v["response"]["meta"].clone();
    obj(&raw)?;
    if raw["type"].is_null() {
        raw["type"] = v["item"]["type"].clone();
    }
    let item = media(&raw)?;
    Ok(json!({"episodes":item["episodes"],"item":item}))
}

pub(super) fn stream_poll(v: &Value) -> Result<Value> {
    let events=array(v,"events")?.iter().map(|e|{let mut out=json!({"sequence":number(e,"seq")?,"source":string(e,"source")?,"sources":array(e,"streams")?.iter().filter_map(|s|source(s).ok()).collect::<Vec<_>>()});if e["error"].is_string(){out["error"]=json!("Source unavailable");}Ok(out)}).collect::<Result<Vec<_>>>()?;
    Ok(json!({"events":events,"done":boolean(v,"done")?}))
}

pub(super) fn live(v: &Value) -> Result<Value> {
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

pub(super) fn live_categories(v: &Value) -> Result<Value> {
    Ok(
        json!({"categories":array(v,"categories")?.iter().map(|c|Ok(json!({"id":id(c,"id")?,"name":string(c,"name")?,"count":number(c,"count")?,"raw":clean(c)}))).collect::<Result<Vec<_>>>()?,"total":v["total"].as_f64().unwrap_or(0.0)}),
    )
}

pub(super) fn guide(v: &Value) -> Result<Value> {
    let programs = v["programs"]
        .as_array()
        .or_else(|| v["programmes"].as_array())
        .or_else(|| v["items"].as_array())
        .ok_or_else(invalid)?;
    Ok(
        json!({"timezone":fallback(v,&["timezone"],""),"timeline":v["timeline"].as_array().into_iter().flatten().map(|t|Ok(json!({"time":number(t,"start")?,"displayTime":fallback(t,&["display_time"],"")}))).collect::<Result<Vec<_>>>()?,"programs":programs.iter().map(|p|{let mut out=json!({"title":fallback(p,&["title"],"No schedule available"),"start":p["start"].as_f64().or_else(||p["start_time"].as_f64()).unwrap_or(0.0),"end":p["end"].as_f64().or_else(||p["end_time"].as_f64()).unwrap_or(0.0),"displayTime":fallback(p,&["display_time"],""),"raw":clean(p)});optional_string(p,&mut out,"description","description");out}).collect::<Vec<_>>()}),
    )
}

pub(super) fn media_array(v: &Value) -> Result<Value> {
    Ok(Value::Array(
        v.as_array()
            .ok_or_else(invalid)?
            .iter()
            .filter_map(|m| media(m).ok())
            .collect(),
    ))
}

pub(super) fn catalogs(v: &Value) -> Result<Value> {
    Ok(Value::Array(
        v.as_array()
            .ok_or_else(invalid)?
            .iter()
            .filter_map(|v| catalog(v).ok())
            .collect(),
    ))
}

pub(super) fn discover(v: &Value) -> Result<Value> {
    let mut out = json!({"items":v["metas"].as_array().or_else(||v["items"].as_array()).or_else(||v["rows"].as_array()).ok_or_else(invalid)?.iter().filter_map(|m|media(m).ok()).collect::<Vec<_>>(),"hasMore":v["has_more"].as_bool().unwrap_or(false)});
    optional_number(v, &mut out, "next_skip", "nextSkip");
    Ok(out)
}

pub(super) fn container(v: &Value) -> Result<Value> {
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
