use super::*;

pub(super) fn request(v: &Value) -> Result {
    Ok({
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
    })
}
