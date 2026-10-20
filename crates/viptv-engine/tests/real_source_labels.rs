//! Only untouched public labels captured from real providers; no generated cases.
use serde_json::{Value, json};
use viptv_engine::stremio::{
    SourcePresentation, source_audio_languages, source_detail, source_quality_height,
};

#[test]
fn actual_provider_labels_match_reviewed_expectations() {
    let capture: Value =
        serde_json::from_str(include_str!("../../../tests/real-source-labels.json")).unwrap();
    for case in capture["cases"].as_array().unwrap() {
        let source = &case["source"];
        let index = &case["capture_index"];
        assert_eq!(
            json!(source_quality_height(source)),
            case["expected_quality_height"],
            "real capture {index}: height"
        );
        let presentation = serde_json::to_value(SourcePresentation::from_stream(source)).unwrap();
        for field in ["video_codec", "container"] {
            if let Some(expected) = case.get(format!("expected_{field}")) {
                assert_eq!(
                    &presentation[field], expected,
                    "real capture {index}: {field}"
                );
            }
        }
        let mut languages = source_audio_languages(source);
        if let Some(expected) = case["expected_audio_languages_set"].as_array() {
            let mut expected: Vec<_> = expected
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            expected.sort();
            languages.sort();
            assert_eq!(languages, expected, "real capture {index}: audio hints");
        }
        if let Some(expected) = case["expected_audio_languages_contains"].as_array() {
            for language in expected {
                assert!(
                    languages.iter().any(|v| v == language.as_str().unwrap()),
                    "real capture {index}: missing audio hint"
                );
            }
        }
        if let Some(expected) = case["expected_detail_contains"].as_array() {
            let detail = source_detail(source).unwrap();
            for text in expected {
                assert!(
                    detail.contains(text.as_str().unwrap()),
                    "real capture {index}: lost title detail"
                );
            }
        }
    }
    eprintln!(
        "REAL_SOURCE_LABELS cases={} synthetic=0",
        capture["cases"].as_array().unwrap().len()
    );
}
