//! Pure cross-platform presentation and request policy. Shells execute effects only.
use crate::CoreError;
use serde_json::{Value, json};
type Result = std::result::Result<Value, CoreError>;
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
fn num(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or(0.0)
}
fn image(v: &Value, k: &str) -> Value {
    v[k].as_str()
        .filter(|s| !s.trim().is_empty())
        .map_or(Value::Null, |s| json!(s))
}
fn watched(v: &Value) -> bool {
    v["watched"]
        .as_bool()
        .unwrap_or(num(v, "duration") > 0.0 && num(v, "position") / num(v, "duration") >= 0.95)
}
fn item_request(v: &Value) -> Value {
    let mut o = json!({});
    for (a, b) in [
        ("id", "id"),
        ("type", "type"),
        ("name", "name"),
        ("poster", "poster"),
        ("year", "year"),
        ("season", "season"),
        ("episode", "episode"),
        ("seriesId", "series_id"),
        ("sourceAddonId", "source_addon_id"),
        ("sourceName", "source_name"),
        ("sourceFingerprint", "source_fingerprint"),
        ("sourceBingeGroup", "source_binge_group"),
        ("sourceReleaseGroup", "source_release_group"),
        ("sourceQuality", "source_quality"),
        ("sourceAudio", "source_audio"),
    ] {
        o[b] = v[a].clone();
    }
    o
}
fn snake(v: &Value) -> Value {
    let mut out = json!({});
    if let Some(obj) = v.as_object() {
        for (k, v) in obj {
            let mut key = String::new();
            for c in k.chars() {
                if c.is_uppercase() {
                    key.push('_');
                    key.extend(c.to_lowercase());
                } else {
                    key.push(c);
                }
            }
            out[&key] = if v.is_object() { snake(v) } else { v.clone() };
        }
    }
    out
}
fn enc(v: &str) -> String {
    url::form_urlencoded::byte_serialize(v.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}
pub fn normalize(kind: &str, v: &Value) -> Result {
    Ok(match kind {
        "presentation" => {
            let m = if v["item"].is_object() { &v["item"] } else { v };
            let episode = num(m, "episode");
            let season = num(m, "season");
            let label = if episode > 0.0 {
                format!(
                    "S{season:.0} E{episode:.0}{}",
                    if text(m, "episodeTitle").is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", text(m, "episodeTitle"))
                    }
                )
            } else {
                String::new()
            };
            let live = text(m, "type") == "live";
            let next = text(m, "queueStatus") == "next";
            let resume = !live && num(m, "position") > 0.0;
            let action = if live {
                "play"
            } else if next {
                "next"
            } else if resume {
                "resume"
            } else if text(m, "type") == "series" && episode <= 0.0 {
                "episodes"
            } else {
                "sources"
            };
            json!({"heroImage":image(m,"background"),"posterImage":image(m,"poster"),"episodeImage":image(m,"thumbnail"),"titleLogo":image(m,"titleLogo"),"title":m["name"],"episodeLabel":label,"progress":if num(m,"duration")>0.0{(num(m,"position")/num(m,"duration")).clamp(0.0,1.0)}else{0.0},"primaryAction":action,"primaryActionLabel":match action{"next"=>"Play next episode","resume"=>"Resume","play"=>"Watch live","episodes"=>"Episodes",_=>"Play"},"resumeEligible":resume,"canAutoNext":!live&&episode>0.0&&num(m,"duration")>0.0&&num(m,"duration")-num(m,"position")<=10.0&&num(m,"position")>0.0})
        }
        "cardPresentation" => {
            let m = &v["item"];
            let queue = text(v, "context") == "queue";
            let live = text(m, "type") == "live";
            let episode = num(m, "episode") > 0.0 || text(m, "type") == "episode";
            let presentation = normalize("presentation", m)?;
            let candidates: &[(&str, &str)] = if live {
                &[("poster", "logo")]
            } else if queue && episode {
                // A series poster is not an episode still. An unavailable still
                // can use an explicitly known landscape, never a portrait crop.
                &[("thumbnail", "episode"), ("background", "landscape")]
            } else {
                &[
                    ("background", "landscape"),
                    ("thumbnail", "landscape"),
                    ("poster", "poster"),
                ]
            };
            let (art, role) = candidates
                .iter()
                .find_map(|(key, role)| {
                    let art = image(m, key);
                    (!art.is_null()).then_some((art, *role))
                })
                .unwrap_or((Value::Null, "none"));
            let status = match text(m, "queueStatus") {
                "next" => "Play next episode".to_owned(),
                "caught_up" => "You're caught up".to_owned(),
                "upcoming" => "Next episode coming soon".to_owned(),
                "pending" | "unavailable" => "Find next episode".to_owned(),
                _ if !live && num(m, "position") > 0.0 => {
                    let seconds = num(m, "position").floor() as u64;
                    format!("Resume at {}:{:02}", seconds / 60, seconds % 60)
                }
                _ => String::new(),
            };
            let mut context = Vec::new();
            if episode {
                context.push(format!(
                    "S{:.0} · E{:.0}",
                    num(m, "season"),
                    num(m, "episode")
                ));
                if !text(m, "episodeTitle").is_empty() {
                    context.push(text(m, "episodeTitle").to_owned());
                }
            }
            if !status.is_empty() {
                context.push(status);
            }
            if context.is_empty() && !live {
                if num(m, "year") > 0.0 {
                    context.push(format!("{:.0}", num(m, "year")));
                }
                if !text(m, "runtime").is_empty() {
                    context.push(text(m, "runtime").to_owned());
                }
                context.extend(
                    m["genres"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .take(2)
                        .map(str::to_owned),
                );
            }
            let action = if live {
                "play"
            } else if !queue {
                "details"
            } else if matches!(
                text(m, "queueStatus"),
                "caught_up" | "upcoming" | "pending" | "unavailable"
            ) {
                "episodes"
            } else {
                text(&presentation, "primaryAction")
            };
            let label = match action {
                "details" => "Details",
                "episodes" => "Episodes",
                _ => text(&presentation, "primaryActionLabel"),
            };
            json!({"image":art,"imageRole":role,"title":m["name"],"subtitle":context.join(" · "),
                "progress":if !live && num(m,"duration")>0.0 { presentation["progress"].clone() } else { Value::Null },
                "primaryAction":action,"primaryActionLabel":label})
        }
        "itemRequest" => item_request(v),
        "playbackRequest" | "preferencesRequest" => snake(v),
        "request" => {
            let op = text(v, "operation");
            let profile = enc(text(v, "profileId"));
            let base = format!("/api/profiles/{profile}");
            let mut body = item_request(&v["item"]);
            let (method, path) = match op {
                "nextEpisode" => ("POST", format!("{base}/continue/next")),
                "saveProgress" => {
                    body["position"] = v["position"].clone();
                    body["duration"] = v["duration"].clone();
                    ("PUT", format!("{base}/progress"))
                }
                "correctProgress" => {
                    body["action"] = v["action"].clone();
                    body["duration"] = v["duration"].clone();
                    ("PUT", format!("{base}/progress/correct"))
                }
                "setQueueVisibility" => {
                    body["hidden"] = v["hidden"].clone();
                    ("PUT", format!("{base}/continue/visibility"))
                }
                "toggleFavorite" => ("POST", format!("{base}/favorites/toggle")),
                "setFavorite" => ("PUT", format!("{base}/favorites")),
                "playback" => {
                    body = snake(&v["playback"]);
                    ("POST", "/api/playback".into())
                }
                "metadata" => {
                    body = Value::Null;
                    let item = &v["item"];
                    let series_id = text(item, "seriesId");
                    let (media_type, media_id) = if series_id.is_empty() {
                        (text(item, "type"), text(item, "id"))
                    } else {
                        ("series", series_id)
                    };
                    (
                        "GET",
                        format!("/api/meta/{}/{}", enc(media_type), enc(media_id)),
                    )
                }
                _ => return Err(CoreError::InvalidInput),
            };
            json!({"method":method,"path":path,"body":body})
        }
        "enrichDetail" => {
            let mut out = v["original"].clone();
            if let Some(m) = v["metadata"].as_object() {
                for (k, value) in m {
                    if !value.is_null()
                        && !(k == "genres" && value.as_array().is_some_and(Vec::is_empty))
                    {
                        out[k] = value.clone();
                    }
                }
            }
            let mut raw = v["original"]["raw"]
                .as_object()
                .cloned()
                .unwrap_or_default();
            if let Some(r) = v["metadata"]["raw"].as_object() {
                raw.extend(r.clone());
            }
            out["raw"] = json!(raw);
            out
        }
        "mergeEpisodeProgress" => {
            let eps = v["episodes"].as_array().ok_or(CoreError::InvalidInput)?;
            let rows = v["history"].as_array().ok_or(CoreError::InvalidInput)?;
            Value::Array(
                eps.iter()
                    .map(|ep| {
                        let row = rows.iter().find(|r| r["id"] == ep["id"]).or_else(|| {
                            rows.iter().find(|r| {
                                r["seriesId"] == v["seriesId"]
                                    && !r["episode"].is_null()
                                    && r["episode"] == ep["episode"]
                                    && r["season"] == ep["season"]
                            })
                        });
                        let mut out = ep.clone();
                        if let Some(row) = row {
                            for key in [
                                "position",
                                "duration",
                                "watched",
                                "sourceAddonId",
                                "sourceName",
                                "sourceFingerprint",
                                "sourceBingeGroup",
                                "sourceReleaseGroup",
                                "sourceQuality",
                                "sourceAudio",
                                "updatedAtMillis",
                            ] {
                                if !row[key].is_null() {
                                    out[key] = row[key].clone();
                                }
                            }
                            out["raw"]["updated_at"] = row["raw"]["updated_at"].clone();
                        }
                        out
                    })
                    .collect(),
            )
        }
        "initialEpisode" => {
            let mut eps = v["episodes"]
                .as_array()
                .ok_or(CoreError::InvalidInput)?
                .clone();
            eps.sort_by(|a, b| {
                num(a, "season")
                    .total_cmp(&num(b, "season"))
                    .then(num(a, "episode").total_cmp(&num(b, "episode")))
            });
            let updated = |e: &Value| {
                e["updatedAtMillis"]
                    .as_f64()
                    .unwrap_or(num(&e["raw"], "updated_at") * 1000.0)
            };
            let latest = eps
                .iter()
                .filter(|e| updated(e) > 0.0)
                .max_by(|a, b| updated(a).total_cmp(&updated(b)));
            if let Some(latest) = latest {
                if watched(latest) {
                    let index = eps.iter().position(|e| e == latest).unwrap();
                    eps.iter()
                        .skip(index + 1)
                        .find(|e| {
                            !watched(e)
                                && e["season"] != 0
                                && e["releasedAtMillis"]
                                    .as_f64()
                                    .or_else(|| {
                                        e["raw"]["released"]
                                            .as_str()
                                            .and_then(|s| {
                                                chrono::DateTime::parse_from_rfc3339(s).ok()
                                            })
                                            .map(|d| d.timestamp_millis() as f64)
                                    })
                                    .unwrap_or(0.0)
                                    <= num(v, "now")
                        })
                        .unwrap_or(latest)
                        .clone()
                } else {
                    latest.clone()
                }
            } else {
                eps.iter()
                    .find(|e| num(e, "position") > 0.0 && !watched(e))
                    .or_else(|| {
                        eps.iter().find(|e| {
                            !v["original"]["episode"].is_null()
                                && e["season"] == v["original"]["season"]
                                && e["episode"] == v["original"]["episode"]
                        })
                    })
                    .or_else(|| eps.iter().find(|e| !watched(e) && e["season"] != 0))
                    .or(eps.first())
                    .cloned()
                    .unwrap_or(Value::Null)
            }
        }
        "exactResume" | "exactResumeSource" => v["sources"]
            .as_array()
            .ok_or(CoreError::InvalidInput)?
            .iter()
            .find(|s| {
                !text(&v["item"], "sourceFingerprint").is_empty()
                    && s.get("sourceFingerprint")
                        .unwrap_or(&s["raw"]["source_fingerprint"])
                        == &v["item"]["sourceFingerprint"]
                    && s["sourceAddonId"] == v["item"]["sourceAddonId"]
            })
            .cloned()
            .unwrap_or(Value::Null),
        "enrichHome" => {
            let mut out = v["original"].clone();
            let m = &v["metadata"];
            for k in [
                "poster",
                "background",
                "description",
                "year",
                "runtime",
                "imdbRating",
                "genres",
                "credits",
            ] {
                if !m[k].is_null() && !m[k].as_array().is_some_and(Vec::is_empty) {
                    out[k] = m[k].clone();
                }
            }
            let is_episode = num(&out, "episode") > 0.0 || text(&out, "type") == "episode";
            if is_episode {
                let matching = m["episodes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|episode| {
                        (!text(&out, "id").is_empty() && text(episode, "id") == text(&out, "id"))
                            || (out["season"].is_number()
                                && out["episode"].is_number()
                                && episode["season"] == out["season"]
                                && episode["episode"] == out["episode"])
                    });
                if let Some(episode) = matching {
                    if !image(episode, "thumbnail").is_null() {
                        out["thumbnail"] = episode["thumbnail"].clone();
                    }
                    if !text(episode, "episodeTitle").is_empty() {
                        out["episodeTitle"] = episode["episodeTitle"].clone();
                    } else if !text(episode, "name").is_empty() {
                        out["episodeTitle"] = episode["name"].clone();
                    }
                }
            } else if !image(m, "thumbnail").is_null() {
                out["thumbnail"] = m["thumbnail"].clone();
            }
            if !image(m, "titleLogo").is_null() {
                out["titleLogo"] = m["titleLogo"].clone();
            }
            if text(&out, "name").is_empty() {
                out["name"] = m["name"].clone();
            }
            // A shelf occurrence needs its matched still, not every episode.
            // Detail responses retain the full typed catalog independently.
            out["episodes"] = json!([]);
            out
        }
        "sourceDisplay" => {
            let opaque = |s: &str| matches(r"^[A-Za-z0-9._-]+:[0-9]+$", s);
            let name = if text(v, "sourceName").trim().is_empty() {
                text(v, "name")
            } else {
                text(v, "sourceName")
            };
            let mut parts = Vec::new();
            for key in ["title", "description", "filename"] {
                let value = text(v, key);
                if !value.trim().is_empty() && !parts.contains(&value) {
                    parts.push(value);
                }
            }
            let description = parts.join("\n");
            let description = description.as_str();
            let provider = text(v, "provider");
            let title = if !name.trim().is_empty() && !opaque(name) {
                name.to_owned()
            } else if !description.trim().is_empty() {
                description
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(180)
                    .collect()
            } else if !provider.trim().is_empty() && !opaque(provider) {
                provider.to_owned()
            } else {
                "Source".into()
            };
            json!({"title":title,"body":if !description.trim().is_empty(){description}else if !opaque(provider){provider}else{""}})
        }
        "sourceIdentity" => {
            let addon = text(v, "addonId");
            let fingerprint = text(v, "fingerprint");
            if addon.is_empty() || fingerprint.is_empty() {
                Value::Null
            } else {
                json!(format!("{addon}\u{0000}{fingerprint}"))
            }
        }
        "resume" => v["sources"]
            .as_array()
            .ok_or(CoreError::InvalidInput)?
            .iter()
            .find(|s| {
                let a = text(s, "sourceAddonId");
                let f = text(s, "sourceFingerprint");
                !a.is_empty()
                    && !f.is_empty()
                    && format!("{a}\u{0000}{f}") == text(v, "expectedIdentity")
            })
            .map(|s| s["id"].clone())
            .unwrap_or(Value::Null),
        "autoNext" => json!(
            v["playing"] == true
                && v["seeking"] != true
                && v["nextAvailable"] == true
                && v["autoplay"] == true
                && matches!(text(v, "type"), "series" | "episode")
                && num(v, "duration") > 10.0
                && num(v, "position") > 0.0
                && num(v, "duration") - num(v, "position") <= 10.0
        ),
        "sourceMatch" => source_match(v),
        "continuationSource" => {
            let sources = v["sources"].as_array().ok_or(CoreError::InvalidInput)?;
            let provider = text(&v["current"], "sourceAddonId");
            let provider = if provider.is_empty() {
                text(v, "outgoingAddonId")
            } else {
                provider
            };
            let found = if !v["current"].is_object() {
                let expected = if provider.is_empty() {
                    text(v, "nextAddonId")
                } else {
                    provider
                };
                sources
                    .iter()
                    .find(|s| !expected.is_empty() && text(s, "sourceAddonId") == expected)
            } else if provider.starts_with("iptv:") {
                sources
                    .iter()
                    .find(|s| text(s, "sourceAddonId") == provider)
            } else {
                sources.iter().filter(|s|text(s,"sourceAddonId").starts_with("addon:")).min_by(|a,b|num(&source_match(&json!({"source":a,"capabilities":v["capabilities"],"preferences":v["preferences"]})),"rank").total_cmp(&num(&source_match(&json!({"source":b,"capabilities":v["capabilities"],"preferences":v["preferences"]})),"rank")))
            };
            found
                .map(|s| {
                    if v["current"].is_object() {
                        s.clone()
                    } else {
                        s["id"].clone()
                    }
                })
                .unwrap_or(Value::Null)
        }
        "catalogFilters" | "catalogDefaults" => {
            let mut filters = v["extras"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|e| e["name"] != "skip")
                .collect::<Vec<_>>();
            if v["supportsSearch"] == true && !filters.iter().any(|f| f["name"] == "search") {
                filters.push(json!({"name":"search","required":false,"options":[]}));
            }
            if kind == "catalogFilters" {
                json!(filters)
            } else {
                let mut out = json!({});
                for f in filters.iter().filter(|f| f["required"] == true) {
                    out[text(f, "name")] = f
                        .get("defaultValue")
                        .filter(|v| !v.is_null())
                        .or_else(|| f["options"].as_array().and_then(|a| a.first()))
                        .cloned()
                        .unwrap_or(json!(""));
                }
                out
            }
        }
        "artworkUrl" => {
            let original = text(v, "original");
            if original.is_empty() {
                Value::Null
            } else {
                let mut uri = original.to_owned();
                if uri.starts_with("https://wsrv.nl/?")
                    && let Ok(u) = url::Url::parse(&uri)
                    && let Some((_, inner)) = u.query_pairs().find(|(k, _)| k == "url")
                {
                    uri = inner.into_owned();
                }
                if matches(
                    r"^https://(image\.tmdb\.org|artworks\.thetvdb\.com|episodes\.metahub\.space|images\.metahub\.space|live\.metahub\.space|assets\.fanart\.tv|i\.imgur\.com)/[^?#@]+$",
                    &uri,
                ) {
                    let width = num(v, "width");
                    let size = if width > 1280.0 {
                        "original"
                    } else if width > 500.0 {
                        "w1280"
                    } else {
                        "w500"
                    };
                    uri = regex::Regex::new(r"^https://image\.tmdb\.org/t/p/(w[0-9]+|original)/")
                        .unwrap()
                        .replace(&uri, format!("https://image.tmdb.org/t/p/{size}/"))
                        .into_owned();
                    json!(format!(
                        "https://wsrv.nl/?url={}&w={}&h={}&fit={}&output={}&q={}&we",
                        enc(&uri),
                        width,
                        num(v, "height"),
                        if v["logo"] == true { "inside" } else { "cover" },
                        if v["logo"] == true { "png" } else { "jpg" },
                        if v["large"] == true { 95 } else { 85 }
                    ))
                } else {
                    json!(original)
                }
            }
        }
        _ => return Err(CoreError::InvalidInput),
    })
}
fn matches(pattern: &str, text: &str) -> bool {
    regex::Regex::new(pattern)
        .expect("policy regex")
        .is_match(text)
}
fn source_match(v: &Value) -> Value {
    let s = &v["source"];
    let caps = &v["capabilities"];
    let prefs = &v["preferences"];
    let mut height = caps["maxHeight"]
        .as_f64()
        .filter(|n| *n > 0.0)
        .unwrap_or(1080.0);
    for (h, q) in [(480.0, "480p"), (720.0, "720p"), (1080.0, "1080p")] {
        if prefs["quality"] == q {
            height = height.min(h);
        }
    }
    let words = [
        text(s, "name"),
        text(s, "title"),
        text(s, "filename"),
        text(s, "audio"),
        text(&s["raw"], "description"),
    ]
    .join("\n")
    .to_lowercase();
    let language = text(prefs, "audioLanguage").to_lowercase();
    let aliases = match language.as_str() {
        "es" | "spa" => "spanish|spa|es",
        "fr" | "fre" | "fra" => "french|fre|fra|fr",
        "de" | "deu" | "ger" => "german|ger|deu|de",
        "it" => "italian|ita|it",
        "pt" | "por" => "portuguese|por|pt",
        "ja" | "jpn" => "japanese|jpn|ja",
        "ko" => "korean|kor|ko",
        "zh" => "chinese|zho|chi|zh",
        "hi" => "hindi|hin|hi",
        "ar" => "arabic|ara|ar",
        _ => "english|eng|en",
    };
    let english = aliases == "english|eng|en";
    let mut mentioned = false;
    let mut explicit = false;
    let mut dubbed = false;
    let mut multi = false;
    for line in words.replace(['|', ';'], "\n").lines() {
        let subtitles = matches(r"(^|[^a-z])(?:subtitles?|subs?|captions?)([^a-z]|$)", line);
        let audio = matches(r"(^|[^a-z])(?:audio|dubbed|dub)([^a-z]|$)", line);
        if subtitles && !audio {
            continue;
        }
        let line = regex::Regex::new(&format!(
            "(?:{aliases})[ ._:-]+(?:subtitles?|subs?|captions?)"
        ))
        .unwrap()
        .replace_all(line, "");
        mentioned |= matches(&format!("(^|[^a-z])(?:{aliases})([^a-z]|$)"), &line);
        explicit |= matches(
            &format!(
                "(^|[^a-z])(?:(?:{aliases})[ ._:-]+(?:audio|dubbed|dub)|(?:audio|dubbed|dub)[ ._:-]+(?:{aliases}))([^a-z]|$)"
            ),
            &line,
        );
        if english {
            dubbed |= matches(r"(^|[^a-z])(?:dubbed|dub)([^a-z]|$)", &line);
            multi |= matches(
                r"(^|[^a-z])(?:dual[ ._-]?audio|multi[ ._-]?audio)([^a-z]|$)",
                &line,
            );
        }
    }
    let reported = s["raw"]["reported_languages"].as_array().is_some_and(|a| {
        a.iter().any(|x| {
            matches(
                &format!("(?i)^(?:{aliases})(?:[-_].*)?$"),
                x.as_str().unwrap_or(""),
            )
        })
    });
    let evidence = if english {
        s["raw"]["audioEvidenceScore"].as_f64()
    } else {
        None
    };
    let score = evidence
        .unwrap_or(
            (i32::from(reported)
                + i32::from(mentioned)
                + 2 * i32::from(multi)
                + 3 * i32::from(dubbed)
                + 4 * i32::from(explicit)) as f64,
        )
        .clamp(0.0, 9.0);
    let mut resolution = 0.0;
    for n in [480, 720, 1080, 2160] {
        if words.contains(&format!("{n}p")) {
            resolution = n as f64;
        }
    }
    if words.contains("4k") {
        resolution = 2160.0;
    }
    let h264 = matches(r"(^|[^a-z0-9])(?:h\.?264|x264|avc)([^a-z0-9]|$)", &words);
    let h265 = matches(r"(^|[^a-z0-9])(?:h\.?265|x265|hevc)([^a-z0-9]|$)", &words);
    let heavy = matches(
        r"(^|[^a-z0-9])(?:hdr|hdr10|dv|10bit|10-bit|hi10p|av1)([^a-z0-9]|$)",
        &words,
    );
    let likely = resolution > 0.0
        && resolution <= height
        && (h264 || (h265 && caps["hevcSdr"] == true))
        && !heavy;
    json!({"rank":(9.0-score)*10.0+if likely&&resolution==height{0.0}else if likely{1.0}else{5.0},"likely":likely,"best":likely&&score>=4.0&&resolution==height})
}
