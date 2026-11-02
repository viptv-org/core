//! Captured anime provider responses through the ranking seam: the audio
//! evidence a provider declares decides the best match for the profile's
//! preferred audio language.
use serde_json::Value;
use viptv_engine::preferences::{
    ProfilePreferences, RankingContext, SourceRanking, fails_hard_filter, language_tier,
    rank_sources,
};
use viptv_engine::stremio::{
    SourceCandidate, SourcePresentation, source_audio_languages, source_detail,
    source_quality_height, source_release_facts, stream_delivery,
};

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/source-ranking-anime-captured.json"
    ))
    .unwrap()
}

fn preferences(language: &str) -> ProfilePreferences {
    ProfilePreferences {
        preferred_audio_language: Some(language.into()),
        ..Default::default()
    }
}

/// Mirrors the candidate `local_engine/sources.rs` builds for an add-on row;
/// core's add-on path cannot take a torrent row, so the row is normalized here.
fn addon_candidate(id: &str, provider: &str, row: &Value) -> SourceCandidate {
    SourceCandidate {
        id: id.into(),
        provider_id: provider.into(),
        label: row["name"].as_str().unwrap_or("Stream").into(),
        detail: source_detail(row),
        quality_height: source_quality_height(row),
        audio_languages: source_audio_languages(row),
        presentation: SourcePresentation::from_stream(row),
        delivery: Some(stream_delivery(row).into()),
        ..Default::default()
    }
    .with_release_facts(source_release_facts(row))
}

fn rows_of(case: &Value) -> Vec<SourceCandidate> {
    let provider = case["provider"].as_str().unwrap();
    let id = case["id"].as_str().unwrap();
    case["streams"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, row)| addon_candidate(&format!("{id}-{index}"), provider, row))
        .collect()
}

fn replay(
    rows: &[SourceCandidate],
    language: &str,
) -> (Vec<u8>, SourceRanking, Vec<SourceCandidate>) {
    let mut ranked = rows.to_vec();
    let tiers = rows
        .iter()
        .map(|row| language_tier(row, Some(language), None))
        .collect();
    let ranking = rank_sources(
        &mut ranked,
        &preferences(language),
        &RankingContext::default(),
    );
    (tiers, ranking, ranked)
}

fn best_row(rows: &[SourceCandidate], ranking: &SourceRanking) -> Option<usize> {
    let best = ranking.best_match_id.as_deref()?;
    rows.iter().position(|row| row.id == best)
}

fn strongest_tier(rows: &[SourceCandidate], tiers: &[u8]) -> Option<u8> {
    rows.iter()
        .zip(tiers)
        .filter(|(row, _)| !fails_hard_filter(row, &Default::default()))
        .map(|(_, tier)| *tier)
        .min()
}

#[test]
fn captured_anime_responses_rank_the_requested_language_dub_first() {
    let corpus = corpus();
    let cases = corpus["addon_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    let mut rows_total = 0;
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let expect = &case["expect"];
        let rows = rows_of(case);
        rows_total += rows.len();
        let (tiers, ranking, _) = replay(&rows, "en");

        let mut histogram = [0usize; 5];
        for tier in &tiers {
            histogram[*tier as usize] += 1;
        }
        let expected: Vec<usize> = expect["tiers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect();
        assert_eq!(&histogram[1..], expected.as_slice(), "{id} language tiers");

        let expected_row = expect["best_row"].as_u64().unwrap() as usize;
        assert_eq!(
            best_row(&rows, &ranking),
            Some(expected_row),
            "{id} best match row"
        );
        assert_eq!(
            tiers[expected_row] as u64,
            expect["best_tier"].as_u64().unwrap(),
            "{id} best match tier"
        );
        assert_eq!(
            rows[expected_row].audio_languages,
            expect["best_languages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect::<Vec<_>>(),
            "{id} best match audio languages"
        );
        assert_eq!(
            rows[expected_row].audio_flags,
            expect["best_flags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect::<Vec<_>>(),
            "{id} best match audio flags"
        );
        assert_eq!(
            Some(tiers[expected_row]),
            strongest_tier(&rows, &tiers),
            "{id} best match from the strongest tier"
        );
        for (row, tier) in rows.iter().zip(&tiers) {
            if *tier == 1 {
                assert!(
                    row.audio_languages
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case("en")),
                    "{id} {} reports a language match without listing it",
                    row.id
                );
            }
        }
    }
    eprintln!(
        "ANIME_RANKING_CORPUS addon_cases={} rows={rows_total} network_requests=0",
        cases.len()
    );
}

#[test]
fn a_different_profile_audio_language_is_respected() {
    let corpus = corpus();
    for case in corpus["addon_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let rows = rows_of(case);
        let (tiers, ranking, _) = replay(&rows, "es");
        for (row, tier) in rows.iter().zip(&tiers) {
            let listed = |language: &str| {
                row.audio_languages
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(language))
            };
            let dual_or_multi = row
                .audio_flags
                .iter()
                .any(|flag| flag == "dual" || flag == "multi");
            let dubbed = row.audio_flags.iter().any(|flag| flag == "dubbed");
            if listed("es") {
                assert_eq!(*tier, 1, "{id} {} lists the requested language", row.id);
            } else if dual_or_multi {
                assert_eq!(
                    *tier, 2,
                    "{id} {} carries a dub beside the original track",
                    row.id
                );
            } else if dubbed && row.audio_languages.is_empty() {
                assert_eq!(
                    *tier, 2,
                    "{id} {} is dubbed but lists no audio language",
                    row.id
                );
            } else if row.audio_languages.is_empty() {
                assert_eq!(*tier, 3, "{id} {} has no audio language evidence", row.id);
            } else {
                assert_eq!(
                    *tier, 4,
                    "{id} {} lists languages that exclude the requested one",
                    row.id
                );
            }
        }
        if let Some(best) = best_row(&rows, &ranking) {
            assert_eq!(
                Some(tiers[best]),
                strongest_tier(&rows, &tiers),
                "{id} best match follows the profile language"
            );
        }
    }
}

#[cfg(feature = "vsources-runtime")]
mod vsources {
    use super::*;
    use viptv_engine::vsources_runtime::{
        CountryCode, Format, SdkStream, candidate as sdk_candidate,
    };

    fn stream(row: &Value) -> SdkStream {
        let format = match row["format"].as_str() {
            Some("mp4") => Format::Mp4,
            _ => Format::Hls,
        };
        let mut stream = SdkStream::new(
            url::Url::parse("https://media.example/master.m3u8").unwrap(),
            format,
        );
        stream.meta.resolution = row["resolution"].as_u64().map(|height| height as u16);
        stream.meta.quality = row["quality"].as_str().map(str::to_owned);
        stream.meta.languages = row["languages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tag| match tag.as_str().unwrap() {
                "Multi" => CountryCode::Multi,
                "En" => CountryCode::En,
                "Ja" => CountryCode::Ja,
                other => panic!("unexpected language tag {other}"),
            })
            .collect();
        stream.meta.dubbed = row["dubbed"].as_bool();
        stream.meta.subbed = row["subbed"].as_bool();
        stream.meta.audio_selection = serde_json::from_value(row["audio_selection"].clone()).ok();
        stream
    }

    fn strings(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn captured_vsources_metadata_keeps_audio_evidence_in_the_requested_language() {
        let corpus = corpus();
        let cases = corpus["vsources_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 20);
        let mut streams_total = 0;
        for case in cases {
            let id = case["id"].as_str().unwrap();
            let expect = &case["expect"];
            let rows: Vec<SourceCandidate> = case["streams"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    streams_total += 1;
                    sdk_candidate(format!("{id}-{index}"), "vsources:1", &stream(row))
                })
                .collect();
            for row in &rows {
                assert_eq!(row.audio_languages, strings(&expect["languages"]), "{id}");
                assert_eq!(row.audio_flags, strings(&expect["flags"]), "{id}");
                assert_eq!(
                    row.subbed,
                    expect["subbed"].as_bool().unwrap(),
                    "{id} subtitle evidence"
                );
                assert_eq!(
                    language_tier(row, Some("en"), None) as u64,
                    expect["tier"].as_u64().unwrap(),
                    "{id} language tier"
                );
            }
            let (tiers, ranking, _) = replay(&rows, "en");
            if let Some(best) = best_row(&rows, &ranking) {
                assert_eq!(
                    Some(tiers[best]),
                    strongest_tier(&rows, &tiers),
                    "{id} best match from the strongest tier"
                );
            }
        }
        eprintln!(
            "ANIME_RANKING_CORPUS vsources_cases={} streams={streams_total} network_requests=0",
            cases.len()
        );
    }
}
