//! SRC-BEST-MATCH-001 ranking keys and language tiers.
use viptv_engine::preferences::{
    AudioCodecSupport, DecodeClass, DecodeSupport, DeviceCapability, HdrPreference, HdrSupport,
    ProfilePreferences, RankingContext, SeriesMemory, VideoCodecSupport, decode_class,
    language_tier, rank_sources,
};
use viptv_engine::stremio::SourceCandidate;

fn source(id: &str) -> SourceCandidate {
    SourceCandidate {
        id: id.into(),
        provider_id: "addon:1".into(),
        label: id.into(),
        quality_height: Some(1080),
        delivery: Some("direct".into()),
        ..Default::default()
    }
}
fn with(id: &str, change: impl FnOnce(&mut SourceCandidate)) -> SourceCandidate {
    let mut candidate = source(id);
    change(&mut candidate);
    candidate
}
fn langs(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}
fn context(original: Option<&str>, device: Option<u32>) -> RankingContext {
    RankingContext {
        original_language: original.map(str::to_owned),
        device: DeviceCapability {
            max_height: device,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn codec_device(codec: &str, support: DecodeSupport, max_height: Option<u32>) -> DeviceCapability {
    DeviceCapability {
        video_codecs: vec![VideoCodecSupport {
            codec: codec.into(),
            support,
            max_height,
        }],
        ..Default::default()
    }
}
fn ranked(
    mut candidates: Vec<SourceCandidate>,
    preferences: &ProfilePreferences,
    original: Option<&str>,
    device: Option<u32>,
) -> Vec<String> {
    rank_sources(&mut candidates, preferences, &context(original, device));
    candidates.into_iter().map(|c| c.id).collect()
}
fn ranked_in(
    mut candidates: Vec<SourceCandidate>,
    preferences: &ProfilePreferences,
    context: &RankingContext,
) -> Vec<String> {
    rank_sources(&mut candidates, preferences, context);
    candidates.into_iter().map(|c| c.id).collect()
}
fn english() -> ProfilePreferences {
    ProfilePreferences {
        preferred_audio_language: Some("en".into()),
        ..Default::default()
    }
}

#[test]
fn language_tier_table() {
    let unmarked = source("unmarked");
    let english = with("english", |c| c.audio_languages = langs(&["en-US"]));
    let dual_english = with("dual-en", |c| {
        c.audio_languages = langs(&["en"]);
        c.audio_flags = langs(&["dual"]);
    });
    let multi = with("multi", |c| c.audio_flags = langs(&["multi"]));
    let dual_french = with("dual-fr", |c| {
        c.audio_languages = langs(&["fr"]);
        c.audio_flags = langs(&["dual", "dubbed"]);
    });
    let dubbed = with("dubbed", |c| c.audio_flags = langs(&["dubbed"]));
    let french = with("french", |c| c.audio_languages = langs(&["fr"]));
    let french_dubbed = with("fr-dub", |c| {
        c.audio_languages = langs(&["fr"]);
        c.audio_flags = langs(&["dubbed"]);
    });
    let rows = [
        &unmarked,
        &english,
        &dual_english,
        &multi,
        &dual_french,
        &dubbed,
        &french,
        &french_dubbed,
    ];
    let tiers = |requested, original| {
        rows.iter()
            .map(|c| language_tier(c, requested, original))
            .collect::<Vec<_>>()
    };
    // English requested, original unknown: dual/multi ties with unmarked.
    assert_eq!(tiers(Some("en"), None), [3, 1, 1, 3, 3, 3, 4, 4]);
    // English requested for a Japanese title: dual/multi and bare dub are tier 2.
    assert_eq!(tiers(Some("en"), Some("ja")), [3, 1, 1, 2, 2, 2, 4, 4]);
    // English requested and English is original: unmarked ties with tier 1,
    // a bare dub is tier 4, dual/multi still beats dubbed.
    assert_eq!(tiers(Some("en"), Some("en")), [1, 1, 1, 2, 2, 4, 4, 4]);
    // "Original language" resolves to the known original.
    assert_eq!(tiers(None, Some("en")), tiers(Some("en"), Some("en")));
    // "Original language" with no metadata: no language ranking at all.
    assert_eq!(tiers(None, None), [1; 8]);
    // French requested for an English title.
    assert_eq!(tiers(Some("fr"), Some("en")), [3, 4, 2, 2, 1, 2, 1, 1]);
}

#[test]
fn torrentio_rows_for_an_english_profile() {
    let rows = vec![
        with("french-only", |c| c.audio_languages = langs(&["fr"])),
        with("multi-audio", |c| c.audio_flags = langs(&["multi"])),
        source("unmarked"),
        with("dual-audio-gb", |c| {
            c.audio_languages = langs(&["en"]);
            c.audio_flags = langs(&["dual"]);
        }),
    ];
    // Original unknown: unmarked and Multi Audio tie and keep input order.
    assert_eq!(
        ranked(rows.clone(), &english(), None, None),
        ["dual-audio-gb", "multi-audio", "unmarked", "french-only"]
    );
    // Japanese original: Multi Audio (likely contains the dub) beats unmarked.
    assert_eq!(
        ranked(rows.clone(), &english(), Some("ja"), None),
        ["dual-audio-gb", "multi-audio", "unmarked", "french-only"]
    );
    // English original: unmarked ties with explicit English.
    assert_eq!(
        ranked(rows, &english(), Some("en"), None),
        ["unmarked", "dual-audio-gb", "multi-audio", "french-only"]
    );
}

#[test]
fn excluded_rows_sink_below_everything() {
    let rows = vec![
        with("cam", |c| {
            c.release_type = Some("cam".into());
            c.audio_languages = langs(&["en"]);
            c.quality_height = Some(2160);
        }),
        with("3d", |c| c.three_d = true),
        with("dead-torrent", |c| {
            c.delivery = Some("torrent".into());
            c.seeders = Some(4);
        }),
        with("unknown-seeders", |c| c.delivery = Some("torrent".into())),
        with("french-480", |c| {
            c.audio_languages = langs(&["fr"]);
            c.quality_height = Some(480);
        }),
    ];
    assert_eq!(
        ranked(rows, &english(), None, None),
        ["unknown-seeders", "french-480", "cam", "3d", "dead-torrent"]
    );
}

#[test]
fn language_precedes_quality_cap_and_resolution() {
    let rows = vec![
        with("fr-2160", |c| {
            c.audio_languages = langs(&["fr"]);
            c.quality_height = Some(2160);
        }),
        with("en-unknown", |c| {
            c.audio_languages = langs(&["en"]);
            c.quality_height = None;
        }),
        with("en-720", |c| {
            c.audio_languages = langs(&["en"]);
            c.quality_height = Some(720);
        }),
        with("en-2160", |c| {
            c.audio_languages = langs(&["en"]);
            c.quality_height = Some(2160);
        }),
    ];
    let mut capped = english();
    capped.max_quality_height = Some(1080);
    assert_eq!(
        ranked(rows.clone(), &capped, None, None),
        ["en-720", "en-unknown", "en-2160", "fr-2160"]
    );
    assert_eq!(
        ranked(rows, &english(), None, None),
        ["en-2160", "en-720", "en-unknown", "fr-2160"]
    );
}

#[test]
fn device_height_caps_quality_with_the_profile_cap() {
    let rows = || {
        [2160, 1440, 1080, 720]
            .into_iter()
            .map(|h| with(&h.to_string(), |c| c.quality_height = Some(h)))
            .collect::<Vec<_>>()
    };
    let any = ProfilePreferences::default();
    assert_eq!(
        ranked(rows(), &any, None, None),
        ["2160", "1440", "1080", "720"]
    );
    // Device only: a 1080p device with "Best available". Over the cap, the
    // closest height to what the device shows comes first.
    assert_eq!(
        ranked(rows(), &any, None, Some(1080)),
        ["1080", "720", "1440", "2160"]
    );
    // Profile only.
    let mut capped = ProfilePreferences::default();
    capped.max_quality_height = Some(1440);
    assert_eq!(
        ranked(rows(), &capped, None, None),
        ["1440", "1080", "720", "2160"]
    );
    // Both: the lower cap wins either way round.
    assert_eq!(
        ranked(rows(), &capped, None, Some(720)),
        ["720", "1080", "1440", "2160"]
    );
    capped.max_quality_height = Some(720);
    assert_eq!(
        ranked(rows(), &capped, None, Some(2160)),
        ["720", "1080", "1440", "2160"]
    );
}

#[test]
fn release_type_order_at_equal_height() {
    let rows = [
        "hdtv",
        "remux",
        "",
        "webrip",
        "bluray",
        "web_dl",
        "other_rip",
        "dvd",
    ]
    .into_iter()
    .map(|kind| {
        with(if kind.is_empty() { "unknown" } else { kind }, |c| {
            c.release_type = (!kind.is_empty()).then(|| kind.into())
        })
    })
    .collect::<Vec<_>>();
    assert_eq!(
        ranked(rows, &english(), None, None),
        [
            "bluray",
            "web_dl",
            "webrip",
            "unknown",
            "remux",
            "hdtv",
            "other_rip",
            "dvd"
        ]
    );
    // Resolution still precedes release type.
    let rows = vec![
        with("web-720", |c| {
            c.release_type = Some("web_dl".into());
            c.quality_height = Some(720);
        }),
        with("remux-1080", |c| c.release_type = Some("remux".into())),
    ];
    assert_eq!(
        ranked(rows, &english(), None, None),
        ["remux-1080", "web-720"]
    );
}

#[test]
fn hdr_preference_any_prefer_avoid() {
    let rows = vec![
        source("sdr"),
        with("dv", |c| c.dynamic_range = langs(&["dv"])),
        with("hlg", |c| c.dynamic_range = langs(&["hlg"])),
    ];
    let mut preferences = english();
    assert_eq!(
        ranked(rows.clone(), &preferences, None, None),
        ["sdr", "dv", "hlg"]
    );
    preferences.hdr_preference = HdrPreference::Prefer;
    assert_eq!(
        ranked(rows.clone(), &preferences, None, None),
        ["dv", "hlg", "sdr"]
    );
    preferences.hdr_preference = HdrPreference::Avoid;
    assert_eq!(ranked(rows, &preferences, None, None), ["sdr", "dv", "hlg"]);
}

#[test]
fn direct_before_torrents_then_seeders() {
    let torrent = |id: &str, seeders: Option<u32>| {
        with(id, |c| {
            c.delivery = Some("torrent".into());
            c.seeders = seeders;
        })
    };
    let rows = vec![
        torrent("t-unknown", None),
        torrent("t-10", Some(10)),
        source("direct"),
        torrent("t-500", Some(500)),
        with("legacy", |c| c.delivery = None),
    ];
    assert_eq!(
        ranked(rows, &english(), None, None),
        ["direct", "legacy", "t-500", "t-10", "t-unknown"]
    );
}

#[test]
fn new_fields_are_omitted_when_unknown_and_old_payloads_parse() {
    let old = serde_json::json!({"id":"a","provider_id":"p","label":"L","detail":null,"quality_height":1080,"audio_languages":["en"],"presentation":null});
    let parsed: SourceCandidate = serde_json::from_value(old.clone()).unwrap();
    assert!(
        parsed.release_type.is_none()
            && !parsed.three_d
            && parsed.delivery.is_none()
            && parsed.video_codec.is_none()
            && parsed.dv_base_layer.is_none()
            && parsed.binge_group.is_none()
            && !parsed.subbed
            && parsed.subtitle_languages.is_empty()
    );
    assert_eq!(serde_json::to_value(&parsed).unwrap(), old);
    let full = with("b", |c| {
        c.release_type = Some("web_dl".into());
        c.three_d = true;
        c.dynamic_range = langs(&["dv", "hdr10"]);
        c.audio_flags = langs(&["dual"]);
        c.subbed = true;
        c.subtitle_languages = langs(&["fr"]);
        c.seeders = Some(7);
        c.delivery = Some("torrent".into());
        c.video_codec = Some("hevc".into());
        c.dv_base_layer = Some("hdr10".into());
        c.binge_group = Some("netflix".into());
    });
    let wire = serde_json::to_value(&full).unwrap();
    for (key, value) in [
        ("release_type", serde_json::json!("web_dl")),
        ("three_d", serde_json::json!(true)),
        ("dynamic_range", serde_json::json!(["dv", "hdr10"])),
        ("audio_flags", serde_json::json!(["dual"])),
        ("subbed", serde_json::json!(true)),
        ("subtitle_languages", serde_json::json!(["fr"])),
        ("seeders", serde_json::json!(7)),
        ("delivery", serde_json::json!("torrent")),
        ("video_codec", serde_json::json!("hevc")),
        ("dv_base_layer", serde_json::json!("hdr10")),
        ("binge_group", serde_json::json!("netflix")),
    ] {
        assert_eq!(wire[key], value, "{key}");
    }
}

#[test]
fn best_matches_skip_hard_filters_and_cap_at_five() {
    let mut rows = vec![
        with("cam", |c| {
            c.release_type = Some("cam".into());
            c.quality_height = Some(2160);
        }),
        with("3d", |c| c.three_d = true),
        with("dead-torrent", |c| {
            c.delivery = Some("torrent".into());
            c.seeders = Some(4);
        }),
        with("unknown-seeders", |c| {
            c.delivery = Some("torrent".into());
            c.seeders = None;
        }),
        source("a"),
        source("b"),
        source("c"),
        source("d"),
        source("e"),
    ];
    let ranking = rank_sources(&mut rows, &ProfilePreferences::default(), &context(None, None));
    assert_eq!(ranking.best_match_ids, ["a", "b", "c", "d", "e"]);
    assert_eq!(ranking.best_match_id.as_deref(), Some("a"));
    // Hard-filtered rows stay in the ranked rows, after every passer.
    assert_eq!(
        rows.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        [
            "a",
            "b",
            "c",
            "d",
            "e",
            "unknown-seeders",
            "cam",
            "3d",
            "dead-torrent"
        ]
    );
}

#[test]
fn best_matches_are_empty_when_every_row_fails_a_hard_filter() {
    let mut rows = vec![
        with("cam", |c| c.release_type = Some("cam".into())),
        with("3d", |c| c.three_d = true),
    ];
    let ranking = rank_sources(&mut rows, &ProfilePreferences::default(), &context(None, None));
    assert!(ranking.best_match_ids.is_empty());
    assert_eq!(ranking.best_match_id, None);
}

#[test]
fn best_match_fully_matches_every_preference() {
    let mut rows = vec![
        source("perfect"),
        with("french", |c| c.audio_languages = langs(&["fr"])),
    ];
    let ranking = rank_sources(&mut rows, &english(), &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("perfect"));
    assert!(ranking.best_match_fully_matches);
}

#[test]
fn best_match_quality_must_be_at_the_cap() {
    let mut capped = english();
    capped.max_quality_height = Some(1080);
    let mut below = vec![with("low", |c| c.quality_height = Some(720))];
    let ranking = rank_sources(&mut below, &capped, &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("low"));
    assert!(!ranking.best_match_fully_matches);
    let mut at_cap = vec![with("high", |c| c.quality_height = Some(1080))];
    let ranking = rank_sources(&mut at_cap, &capped, &context(None, None));
    assert!(ranking.best_match_fully_matches);
}

#[test]
fn best_match_language_disqualifies_only_a_reported_differing_language() {
    let mut french = vec![with("french", |c| c.audio_languages = langs(&["fr"]))];
    let ranking = rank_sources(&mut french, &english(), &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("french"));
    assert!(!ranking.best_match_fully_matches);
    let mut unmarked = vec![source("unmarked")];
    let ranking = rank_sources(&mut unmarked, &english(), &context(Some("ja"), None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("unmarked"));
    assert!(ranking.best_match_fully_matches);
}

#[test]
fn best_match_hdr_preference_must_be_met() {
    let pair = || {
        vec![
            source("sdr"),
            with("dv", |c| c.dynamic_range = langs(&["dv"])),
        ]
    };
    let mut prefer = english();
    prefer.hdr_preference = HdrPreference::Prefer;
    let mut rows = pair();
    let ranking = rank_sources(&mut rows, &prefer, &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("dv"));
    assert!(ranking.best_match_fully_matches);
    let mut avoid = english();
    avoid.hdr_preference = HdrPreference::Avoid;
    let mut rows = pair();
    let ranking = rank_sources(&mut rows, &avoid, &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("sdr"));
    assert!(ranking.best_match_fully_matches);
}

#[test]
fn best_match_never_fully_matches_a_torrent() {
    let mut rows = vec![with("torrent", |c| c.delivery = Some("torrent".into()))];
    let ranking = rank_sources(&mut rows, &ProfilePreferences::default(), &context(None, None));
    assert_eq!(ranking.best_match_id.as_deref(), Some("torrent"));
    assert!(!ranking.best_match_fully_matches);
}

#[test]
fn best_match_fully_matches_is_false_without_a_best_match() {
    let mut rows = vec![with("cam", |c| c.release_type = Some("cam".into()))];
    let ranking = rank_sources(&mut rows, &ProfilePreferences::default(), &context(None, None));
    assert_eq!(ranking.best_match_id, None);
    assert!(!ranking.best_match_fully_matches);
}

#[test]
fn decode_class_reads_the_reported_capability() {
    let hevc = with("hevc", |c| c.video_codec = Some("hevc".into()));
    assert_eq!(
        decode_class(&hevc, &DeviceCapability::default()),
        DecodeClass::Unknown
    );
    assert_eq!(
        decode_class(
            &hevc,
            &codec_device("HEVC", DecodeSupport::Supported, Some(2160))
        ),
        DecodeClass::Supported
    );
    assert_eq!(
        decode_class(
            &hevc,
            &codec_device("hevc", DecodeSupport::Unsupported, None)
        ),
        DecodeClass::Unsupported
    );
}

#[test]
fn decode_class_orders_supported_unknown_risky_and_filters_unsupported() {
    let rows = || {
        vec![
            with("unsupported", |c| {
                c.video_codec = Some("av1".into());
                c.quality_height = Some(2160);
            }),
            with("risky", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["dv"]);
                c.dv_base_layer = Some("none".into());
            }),
            source("unknown-codec"),
            with("supported", |c| c.video_codec = Some("hevc".into())),
        ]
    };
    let device = DeviceCapability {
        video_codecs: vec![
            VideoCodecSupport {
                codec: "av1".into(),
                support: DecodeSupport::Unsupported,
                max_height: None,
            },
            VideoCodecSupport {
                codec: "hevc".into(),
                support: DecodeSupport::Supported,
                max_height: Some(2160),
            },
        ],
        dolby_vision: DecodeSupport::Unsupported,
        ..Default::default()
    };
    let context = RankingContext {
        device: device.clone(),
        ..Default::default()
    };
    let mut candidates = rows();
    let ranking = rank_sources(&mut candidates, &ProfilePreferences::default(), &context);
    assert_eq!(
        candidates.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["supported", "unknown-codec", "risky", "unsupported"]
    );
    assert_eq!(ranking.best_match_ids, ["supported", "unknown-codec", "risky"]);
}

#[test]
fn dolby_vision_without_a_base_layer_is_risky_only_without_a_decoder() {
    let rows = || {
        vec![
            with("dv-only", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["dv"]);
            }),
            with("dv-hdr10", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["dv", "hdr10"]);
                c.dv_base_layer = Some("hdr10".into());
            }),
            with("hdr10", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["hdr10"]);
            }),
        ]
    };
    let without = RankingContext {
        device: DeviceCapability {
            dolby_vision: DecodeSupport::Unsupported,
            ..codec_device("hevc", DecodeSupport::Supported, Some(2160))
        },
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &ProfilePreferences::default(), &without),
        ["dv-hdr10", "hdr10", "dv-only"]
    );
    // A decoder, or no answer, keeps the DV row with the other HDR rows.
    let with_decoder = RankingContext {
        device: DeviceCapability {
            dolby_vision: DecodeSupport::Supported,
            ..codec_device("hevc", DecodeSupport::Supported, Some(2160))
        },
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &ProfilePreferences::default(), &with_decoder),
        ["dv-only", "dv-hdr10", "hdr10"]
    );
}

#[test]
fn codec_decoded_only_in_software_at_this_resolution_is_risky() {
    let rows = || {
        vec![
            with("hevc-2160", |c| {
                c.video_codec = Some("hevc".into());
                c.quality_height = Some(2160);
            }),
            with("hevc-1080", |c| {
                c.video_codec = Some("hevc".into());
                c.quality_height = Some(1080);
            }),
            with("hevc-unknown-height", |c| {
                c.video_codec = Some("hevc".into());
                c.quality_height = None;
            }),
        ]
    };
    let hardware_1080 = RankingContext {
        device: codec_device("hevc", DecodeSupport::Supported, Some(1080)),
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &ProfilePreferences::default(), &hardware_1080),
        ["hevc-1080", "hevc-unknown-height", "hevc-2160"]
    );
    // A codec the device cannot decode at all is a hard filter.
    let undecodable = RankingContext {
        device: DeviceCapability {
            max_height: Some(1080),
            ..codec_device("hevc", DecodeSupport::Unsupported, Some(1080))
        },
        ..Default::default()
    };
    let mut candidates = rows();
    let ranking = rank_sources(&mut candidates, &ProfilePreferences::default(), &undecodable);
    assert_eq!(
        candidates.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["hevc-1080", "hevc-unknown-height", "hevc-2160"]
    );
    assert!(ranking.best_match_ids.is_empty());
}

#[test]
fn series_memory_ranks_first_within_its_language_tier() {
    let rows = || {
        vec![
            with("other-group", |c| c.binge_group = Some("g2".into())),
            with("remembered", |c| {
                c.binge_group = Some("g1".into());
                c.quality_height = Some(720);
            }),
            with("other-provider", |c| {
                c.provider_id = "addon:2".into();
                c.binge_group = Some("g1".into());
            }),
            source("no-group"),
            with("remembered-french", |c| {
                c.binge_group = Some("g1".into());
                c.audio_languages = langs(&["fr"]);
            }),
        ]
    };
    let memory = SeriesMemory {
        provider_id: "addon:1".into(),
        binge_group: "g1".into(),
    };
    let with_memory = RankingContext {
        series_memory: Some(memory),
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &english(), &with_memory),
        [
            "remembered",
            "other-group",
            "other-provider",
            "no-group",
            "remembered-french"
        ]
    );
    // Without memory the rows keep their quality order.
    assert_eq!(
        ranked_in(rows(), &english(), &RankingContext::default()),
        [
            "other-group",
            "other-provider",
            "no-group",
            "remembered",
            "remembered-french"
        ]
    );
}

#[test]
fn hdr_preference_prefers_only_decodable_dolby_vision() {
    let rows = || {
        vec![
            with("dv-only", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["dv"]);
            }),
            with("dv-hdr10", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["dv", "hdr10"]);
                c.dv_base_layer = Some("hdr10".into());
            }),
            with("hdr10", |c| {
                c.video_codec = Some("hevc".into());
                c.dynamic_range = langs(&["hdr10"]);
            }),
            source("sdr"),
        ]
    };
    let mut prefer = english();
    prefer.hdr_preference = HdrPreference::Prefer;
    let no_dv_decoder = RankingContext {
        device: DeviceCapability {
            dolby_vision: DecodeSupport::Unsupported,
            ..codec_device("hevc", DecodeSupport::Supported, Some(2160))
        },
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &prefer, &no_dv_decoder),
        ["dv-hdr10", "hdr10", "sdr", "dv-only"]
    );
    let dv_decoder = RankingContext {
        device: DeviceCapability {
            dolby_vision: DecodeSupport::Supported,
            ..codec_device("hevc", DecodeSupport::Supported, Some(2160))
        },
        ..Default::default()
    };
    assert_eq!(
        ranked_in(rows(), &prefer, &dv_decoder),
        ["dv-only", "dv-hdr10", "hdr10", "sdr"]
    );
}

#[test]
fn capability_names_validate_and_unknown_display_facts_are_carried() {
    let device = DeviceCapability {
        max_height: Some(1080),
        video_codecs: vec![VideoCodecSupport {
            codec: "hevc".into(),
            support: DecodeSupport::Supported,
            max_height: Some(2160),
        }],
        dolby_vision: DecodeSupport::Unknown,
        display_hdr: vec![HdrSupport {
            hdr: "hdr10".into(),
            support: DecodeSupport::Supported,
        }],
        audio_codecs: vec![AudioCodecSupport {
            codec: "eac3".into(),
            support: DecodeSupport::Unsupported,
        }],
    };
    assert!(device.validate().is_ok());
    let wire = serde_json::to_value(&RankingContext {
        device: device.clone(),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(wire["device"]["display_hdr"][0]["hdr"], "hdr10");
    assert_eq!(wire["device"]["audio_codecs"][0]["codec"], "eac3");
    for invalid in [
        DeviceCapability {
            max_height: Some(100),
            ..device.clone()
        },
        DeviceCapability {
            video_codecs: vec![VideoCodecSupport {
                codec: "hevc".into(),
                support: DecodeSupport::Supported,
                max_height: Some(4321),
            }],
            ..Default::default()
        },
        DeviceCapability {
            audio_codecs: vec![AudioCodecSupport {
                codec: String::new(),
                support: DecodeSupport::Unknown,
            }],
            ..Default::default()
        },
    ] {
        assert!(invalid.validate().is_err());
    }
    let memory = RankingContext {
        series_memory: Some(SeriesMemory {
            provider_id: String::new(),
            binge_group: "g1".into(),
        }),
        ..Default::default()
    };
    assert!(memory.validate().is_err());
}
