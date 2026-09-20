use super::*;

pub(super) fn auto_next(v: &Value) -> Value {
    json!(
        v["playing"] == true
            && v["seeking"] != true
            && v["nextAvailable"] == true
            && v["autoplay"] == true
            && matches!(text(v, "type"), "series" | "episode")
            && num(v, "duration") > 10.0
            && num(v, "position") > 0.0
            && num(v, "duration") - num(v, "position") <= 10.0
    )
}

pub(super) fn source_display(v: &Value) -> Value {
    {
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
}

pub(super) fn source_identity(v: &Value) -> Value {
    {
        let addon = text(v, "addonId");
        let fingerprint = text(v, "fingerprint");
        if addon.is_empty() || fingerprint.is_empty() {
            Value::Null
        } else {
            json!(format!("{addon}\u{0000}{fingerprint}"))
        }
    }
}

pub(super) fn continuation_source(v: &Value) -> Result {
    Ok({
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
    })
}

pub(super) fn source_match(v: &Value) -> Value {
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
