use viptv_engine::presentation::{AddonFailure, CatalogPage, DetailSnapshot};

#[test]
fn page_failures_default_empty_and_round_trip() {
    let page: CatalogPage = serde_json::from_value(serde_json::json!({
        "cards": [],
        "next_cursor": null,
        "complete": true,
    }))
    .unwrap();
    assert!(page.failures.is_empty());

    let with_failures = CatalogPage {
        cards: vec![],
        next_cursor: None,
        complete: true,
        failures: vec![AddonFailure {
            addon_id: 7,
            addon_name: "Broken".into(),
            error_code: "addon_unavailable".into(),
        }],
    };
    let round_tripped: CatalogPage =
        serde_json::from_value(serde_json::to_value(&with_failures).unwrap()).unwrap();
    assert_eq!(round_tripped.failures.len(), 1);
    assert_eq!(round_tripped.failures[0].addon_id, 7);
    assert_eq!(round_tripped.failures[0].addon_name, "Broken");
    assert_eq!(round_tripped.failures[0].error_code, "addon_unavailable");
}

#[test]
fn detail_failures_default_empty() {
    let detail: DetailSnapshot = serde_json::from_value(serde_json::json!({
        "item": {
            "identity": {"id": "tt1", "kind": "movie", "ids": []},
            "provider_id": "addon:a",
            "title": "Title",
            "subtitle": null,
            "description": null,
            "poster": null,
            "landscape": null,
            "logo": null,
            "progress": null,
            "badges": []
        },
        "episodes": [],
        "next_cursor": null,
    }))
    .unwrap();
    assert!(detail.failures.is_empty());
}