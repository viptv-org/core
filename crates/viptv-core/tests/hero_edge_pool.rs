use serde_json::{Value, json};
use viptv_core::normalize;
use viptv_core::policy::hero::{HeroEdgePool, HeroEdgePoolInput, hero_edge_pool};

fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../tests/hero-edge-pool-vectors.json")).unwrap()
}

#[test]
fn shared_hero_edge_vectors_match_the_json_bridge_and_typed_api() {
    let vectors = vectors();
    assert!(vectors.len() >= 30);
    for vector in vectors {
        let name = vector["name"].as_str().unwrap();
        let bridged: Value = serde_json::from_str(
            &normalize(
                "heroEdgePool".into(),
                vector["input"].to_string(),
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(bridged, vector["expected"], "{name}");
        let input: HeroEdgePoolInput = serde_json::from_value(vector["input"].clone()).unwrap();
        let expected: HeroEdgePool = serde_json::from_value(vector["expected"].clone()).unwrap();
        assert_eq!(hero_edge_pool(&input), expected, "{name}");
    }
}

#[test]
fn hero_edge_bridge_requires_media_type_and_rejects_mistyped_facts() {
    for input in [
        json!({"genres": [], "availableEdges": []}),
        json!({"mediaType": null, "genres": [], "availableEdges": []}),
        json!({"mediaType": "movie", "genres": "Horror", "availableEdges": []}),
        json!({"mediaType": "movie", "genres": null, "availableEdges": []}),
        json!({"mediaType": "movie", "genres": [null], "availableEdges": []}),
    ] {
        assert!(
            normalize("heroEdgePool".into(), input.to_string(), String::new()).is_err(),
            "{input}"
        );
    }
}
