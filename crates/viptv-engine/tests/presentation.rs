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

#[test]
fn the_season_index_reads_an_absent_season_as_season_one_and_keeps_uncoordinated_cards_last() {
    use viptv_engine::identity::EpisodeCoordinates;
    use viptv_engine::presentation::{canonical_episodes, episode_seasons};
    let mut cards = viptv_engine::stremio::cards(
        &serde_json::json!({"metas":[
            {"id":"explicit","type":"series","name":"Explicit season two"},
            {"id":"absent","type":"series","name":"Absent season"},
            {"id":"dead","type":"series","name":"No number"}]})
        .to_string(),
        "addon",
    )
    .unwrap();
    cards[0].identity.episode = Some(EpisodeCoordinates {
        order: "aired".into(),
        season: Some(2),
        episode: 1,
    });
    cards[1].identity.episode = Some(EpisodeCoordinates {
        order: "aired".into(),
        season: None,
        episode: 4,
    });
    let cards = canonical_episodes(cards);
    assert_eq!(cards[0].identity.id, "absent");
    assert_eq!(cards[1].identity.id, "explicit");
    assert_eq!(cards[2].identity.id, "dead");
    assert_eq!(
        episode_seasons(&cards)
            .iter()
            .map(|season| (season.season, season.episode_count, season.offset))
            .collect::<Vec<_>>(),
        [(Some(1), 1, 0), (Some(2), 1, 1), (None, 1, 2)]
    );
}
