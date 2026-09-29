use super::*;

pub(super) fn request(v: &Value) -> Result {
    Ok({
        let op = text(v, "operation");
        let profile = enc(text(v, "profileId"));
        let base = format!("/api/profiles/{profile}");
        let mut body = item_request(&v["item"]);
        let (method, path) = match op {
            "nextEpisode" => ("POST", format!("{base}/continue/next")),
            "sources" => ("POST", "/api/streams".into()),
            "sourcesV2" => ("POST", "/api/v2/streams".into()),
            "sourcesPoll" | "sourcesPollV2" => {
                body = Value::Null;
                let id = text(v, "id");
                if id.is_empty() {
                    return Err(CoreError::InvalidInput);
                }
                let after = v["after"].as_u64().unwrap_or(0);
                let prefix = if op == "sourcesPollV2" {
                    "/api/v2/streams"
                } else {
                    "/api/streams"
                };
                ("GET", format!("{prefix}/{}?after={after}", enc(id)))
            }
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
            "playbackV2" => {
                body = playback_v2_request(&v["playback"])?;
                ("POST", "/api/v2/playback".into())
            }
            "playbackV2Status" | "playbackV2Heartbeat" | "playbackV2Stop" => {
                let id = text(v, "id");
                if id.is_empty()
                    || id.len() > 128
                    || !id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
                {
                    return Err(CoreError::InvalidInput);
                }
                body = Value::Null;
                let path = format!("/api/v2/playback/{}", enc(id));
                match op {
                    "playbackV2Heartbeat" => {
                        body = json!({});
                        ("POST", format!("{path}/heartbeat"))
                    }
                    "playbackV2Stop" => ("DELETE", path),
                    _ => ("GET", path),
                }
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
    })
}

fn playback_v2_request(input: &Value) -> Result {
    let request: crate::dto::PlaybackV2Request =
        serde_json::from_value(input.clone()).map_err(|_| CoreError::InvalidInput)?;
    if request.request_id.is_empty()
        || request.request_id.len() > 128
        || !request
            .request_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        || request.stream_id.is_empty()
        || request.stream_id.len() > 128
        || !(0.0..=604800.0).contains(&request.position)
        || !(2..=16384).contains(&request.client.max_width)
        || !(2..=16384).contains(&request.client.max_height)
        || request.audio_track.is_some_and(|n| n > 65535)
        || request.subtitle_track.is_some_and(|n| n > 65535)
        || [&request.client.video_codecs, &request.client.audio_codecs]
            .iter()
            .any(|list| {
                list.is_empty()
                    || list.len() > 16
                    || list.iter().any(|codec| {
                        codec.is_empty()
                            || codec.len() > 32
                            || !codec
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    })
            })
    {
        return Err(CoreError::InvalidInput);
    }
    serde_json::to_value(request)
        .map(|v| snake(&v))
        .map_err(|_| CoreError::InvalidInput)
}
