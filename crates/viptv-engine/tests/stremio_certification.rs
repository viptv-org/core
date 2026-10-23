use serde_json::json;
use viptv_engine::stremio;

#[test]
fn certification_survives_title_and_episode_projection_without_inventing_unknown_values() {
    let raw = json!({"meta":{"id":"series","type":"series","name":"Series","contentRating":"TV-Y","certification":"R","videos":[
        {"id":"safe","title":"Safe","season":1,"episode":1,"contentRating":"TV-Y"},
        {"id":"adult","title":"Adult","season":1,"episode":2,"contentRating":"TV-MA"},
        {"id":"alternate","title":"Alternate field","season":1,"episode":3,"certification":"PG-13"},
        {"id":"unknown","title":"Unknown","season":1,"episode":4}
    ]}});
    let detail =
        stremio::detail_page(&raw.to_string(), "addon:fixture", "series", "series", 0, 24).unwrap();
    assert_eq!(
        detail.item.metadata.unwrap().certification.as_deref(),
        Some("TV-Y")
    );
    let ratings = detail
        .episodes
        .iter()
        .map(|card| {
            card.metadata
                .as_ref()
                .and_then(|meta| meta.certification.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ratings,
        vec![Some("TV-Y"), Some("TV-MA"), Some("PG-13"), None]
    );
}

#[test]
fn certification_is_bounded_declared_text_not_a_rating_guess() {
    let cards=stremio::cards(&json!({"metas":[
        {"id":"fallback","type":"movie","name":"Fallback","contentRating":{},"certification":"PG"},
        {"id":"long","type":"movie","name":"Long","contentRating":"x".repeat(129)},
        {"id":"missing","type":"movie","name":"Missing","genres":["Family"]}
    ]}).to_string(),"addon:fixture").unwrap();
    assert_eq!(
        cards[0].metadata.as_ref().unwrap().certification.as_deref(),
        Some("PG")
    );
    assert!(cards[1].metadata.as_ref().unwrap().certification.is_none());
    assert!(cards[2].metadata.as_ref().unwrap().certification.is_none());
}
