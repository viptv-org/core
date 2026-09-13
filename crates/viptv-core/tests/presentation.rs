use serde_json::{Value, json};
fn call(kind: &str, input: Value) -> Value {
    serde_json::from_str(
        &viptv_core::normalize(kind.into(), input.to_string(), "https://example.com".into())
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn hero_never_promotes_portrait_and_home_enrichment_preserves_progress() {
    let item = json!({"id":"episode","type":"series","name":"Title","poster":"portrait.jpg","position":42,"duration":100,"sourceFingerprint":"saved"});
    let view = call("presentation", item.clone());
    assert!(view["heroImage"].is_null());
    assert_eq!(view["posterImage"], "portrait.jpg");
    let enriched = call(
        "enrichHome",
        json!({"original":item,"metadata":{"background":"landscape.jpg","position":0,"sourceFingerprint":"wrong"}}),
    );
    assert_eq!(enriched["position"], 42);
    assert_eq!(enriched["sourceFingerprint"], "saved");
    assert_eq!(call("presentation", enriched)["heroImage"], "landscape.jpg");
}
#[test]
fn malformed_rows_do_not_blank_valid_items() {
    let out = call(
        "discover",
        json!({"items":[{"id":"x","type":"movie","name":"Movie"},{"type":"bad"}]}),
    );
    assert_eq!(out["items"].as_array().unwrap().len(), 1);
}
#[test]
fn playback_request_never_introduces_profile_identity() {
    let out = call(
        "request",
        json!({"operation":"playback","profileId":"private","playback":{"streamId":"opaque","capabilities":{"directPlay":true,"maxWidth":1920}}}),
    );
    assert_eq!(out["path"], "/api/playback");
    assert!(out["body"].get("profile_id").is_none());
    assert_eq!(out["body"]["capabilities"]["direct_play"], true);
}

#[test]
fn episode_metadata_request_targets_its_parent_series() {
    let out = call(
        "request",
        json!({
            "operation":"metadata",
            "item":{
                "id":"bleach:1:3",
                "type":"episode",
                "seriesId":"bleach"
            }
        }),
    );
    assert_eq!(out["method"], "GET");
    assert_eq!(out["path"], "/api/meta/series/bleach");
}
#[test]
fn exact_resume_does_not_cross_provider() {
    let sources = json!([{"id":"bad","sourceAddonId":"iptv:2","sourceFingerprint":"same"},{"id":"good","sourceAddonId":"iptv:1","sourceFingerprint":"same"}]);
    let out = call(
        "exactResume",
        json!({"sources":sources,"item":{"sourceAddonId":"iptv:1","sourceFingerprint":"same"}}),
    );
    assert_eq!(out["id"], "good");
}
#[test]
fn subtitles_are_not_audio_evidence() {
    let caps = json!({"maxHeight":1080,"hevcSdr":false});
    let prefs = json!({"audioLanguage":"en"});
    let sub = call(
        "sourceMatch",
        json!({"source":{"name":"1080p h264 English subtitles","raw":{}},"capabilities":caps,"preferences":prefs}),
    );
    let dub = call(
        "sourceMatch",
        json!({"source":{"name":"1080p h264 English audio","raw":{}},"capabilities":caps,"preferences":prefs}),
    );
    assert!(sub["rank"].as_f64().unwrap() > dub["rank"].as_f64().unwrap());
}

#[test]
fn home_hero_primary_matches_each_contract_state() {
    for (item, action, label) in [
        (
            json!({"type":"live","position":50,"queueStatus":"next"}),
            "play",
            "Watch live",
        ),
        (
            json!({"type":"series","episode":2,"queueStatus":"next","position":50}),
            "next",
            "Play next episode",
        ),
        (json!({"type":"movie","position":50}), "resume", "Resume"),
        (
            json!({"type":"series","episode":2,"position":50}),
            "resume",
            "Resume",
        ),
        (json!({"type":"series"}), "episodes", "Episodes"),
        (json!({"type":"series","episode":2}), "sources", "Play"),
        (json!({"type":"episode","episode":2}), "sources", "Play"),
        (json!({"type":"movie"}), "sources", "Play"),
    ] {
        let view = call("presentation", item);
        assert_eq!(view["primaryAction"], action);
        assert_eq!(view["primaryActionLabel"], label);
    }
}

#[test]
fn source_projection_preserves_identity_and_rich_native_labels() {
    let source = call(
        "source",
        json!({"id":"stream-a","provider":"iptv:4","source_name":"Evening News","title":"HD broadcast","filename":"evening-news.mkv","source_addon_id":"addon:one","source_fingerprint":"fp-a"}),
    );
    assert_eq!(source["name"], "HD broadcast");
    let display = call("sourceDisplay", source.clone());
    assert_eq!(display["title"], "Evening News");
    assert_eq!(display["body"], "HD broadcast\nevening-news.mkv");
    assert_eq!(source["sourceFingerprint"], "fp-a");
    assert_eq!(
        call(
            "sourceDisplay",
            json!({"name":"Label","description":"Same","title":"Same","filename":"Same"})
        )["body"],
        "Same"
    );
}
