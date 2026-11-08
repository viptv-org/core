use viptv_engine::{
    preferences::{self, *},
    stremio::{SourceCandidate, SourcePresentation},
};
fn row(id: &str, addon: &str, supplier: &str, languages: &[&str]) -> SourceCandidate {
    let mut row = SourceCandidate {
        id: id.into(),
        provider_id: addon.into(),
        audio_languages: languages.iter().map(|s| s.to_string()).collect(),
        quality_height: Some(1080),
        ..Default::default()
    };
    row.presentation = Some(SourcePresentation {
        provider_name: Some(supplier.into()),
        ..Default::default()
    });
    row
}
fn memory(languages: &[&str]) -> RememberedSource {
    RememberedSource {
        provider_id: "addon:manual".into(),
        source_id: None,
        provider_name: Some("CineStream".into()),
        audio_languages: languages.iter().map(|s| s.to_string()).collect(),
    }
}
#[test]
fn reused_supplier_and_addon_precede_other_ranked_rows_without_changing_rank_order() {
    let rows = vec![
        row("global-best", "addon:other", "Other", &["en"]),
        row("same-addon", "addon:manual", "Another supplier", &["en"]),
        row("remembered", "addon:manual", "CineStream", &["eng"]),
        row("wrong-audio", "addon:manual", "CineStream", &["ja"]),
    ];
    let result = reusable_sources(
        &rows,
        &memory(&["en"]),
        &ProfilePreferences::default(),
        &RankingContext::default(),
    )
    .unwrap();
    assert_eq!(result.ids, vec!["remembered", "same-addon", "global-best"]);
    assert!(result.preferred);
    assert!(result.fully_matches);
    assert_eq!(rows[0].id, "global-best");
}
#[test]
fn no_same_language_and_unknown_audio_open_the_picker_instead_of_substituting() {
    let rows = vec![
        row("unknown", "addon:manual", "CineStream", &[]),
        row("japanese", "addon:manual", "CineStream", &["ja"]),
    ];
    assert!(
        reusable_sources(
            &rows,
            &memory(&["en"]),
            &ProfilePreferences::default(),
            &RankingContext::default()
        )
        .unwrap()
        .ids
        .is_empty()
    );
}
#[test]
fn missing_supplier_can_fall_back_in_language_but_unknown_language_stays_on_the_addon() {
    let rows = vec![
        row("other-addon", "addon:other", "Other", &["en"]),
        row("same-addon", "addon:manual", "Other", &[]),
    ];
    let known = reusable_sources(
        &rows,
        &memory(&["eng-US"]),
        &ProfilePreferences::default(),
        &RankingContext::default(),
    )
    .unwrap();
    assert_eq!(known.ids, vec!["other-addon"]);
    assert!(!known.preferred);
    assert_eq!(
        reusable_sources(
            &rows,
            &memory(&[]),
            &ProfilePreferences::default(),
            &RankingContext::default()
        )
        .unwrap()
        .ids,
        vec!["same-addon"]
    );
}
#[test]
fn automatic_candidates_keep_hard_filters_even_for_remembered_sources() {
    let mut cam = row("cam", "addon:manual", "CineStream", &["en"]);
    cam.release_type = Some("cam".into());
    let mut low = row("low-seeds", "addon:manual", "CineStream", &["en"]);
    low.delivery = Some("torrent".into());
    low.seeders = Some(2);
    let rows = vec![cam, low, row("good", "addon:other", "Other", &["en"])];
    assert_eq!(
        preferences::automatic_source_ids(&rows, &DeviceCapability::default()),
        vec!["good"]
    );
    assert_eq!(
        reusable_sources(
            &rows,
            &memory(&["en"]),
            &ProfilePreferences::default(),
            &RankingContext::default()
        )
        .unwrap()
        .ids,
        vec!["good"]
    );
}
#[test]
fn disabled_reuse_and_malformed_memories_never_supply_an_automatic_target() {
    let rows = vec![row("good", "addon:manual", "CineStream", &["en"])];
    let prefs = ProfilePreferences {
        remembered_source_reuse: false,
        ..Default::default()
    };
    assert!(
        reusable_sources(&rows, &memory(&["en"]), &prefs, &RankingContext::default())
            .unwrap()
            .ids
            .is_empty()
    );
    let mut bad = memory(&["en"]);
    bad.provider_id = "https://private.invalid/token".into();
    assert!(bad.validate().is_err());
    assert!(memory(&["not a language"]).validate().is_err());
}
#[test]
fn cross_title_reuse_applies_only_without_a_title_choice_and_never_overrides_its_language() {
    let rows = vec![row("english", "addon:manual", "CineStream", &["en"])];
    let title = memory(&["ja"]);
    let global = memory(&["en"]);
    let preferences = ProfilePreferences::default();
    let context = RankingContext::default();
    assert!(
        reusable_source_context(&rows, Some(&title), Some(&global), &preferences, &context)
            .unwrap()
            .ids
            .is_empty()
    );
    assert_eq!(
        reusable_source_context(&rows, None, Some(&global), &preferences, &context)
            .unwrap()
            .ids,
        vec!["english"]
    );
}

#[test]
fn preferred_discovery_uses_current_language_and_stable_supplier_without_soft_match_waits() {
 let mut prefs=ProfilePreferences::default();prefs.preferred_audio_language=Some("es".into());prefs.max_quality_height=Some(1080);
 let remembered=memory(&["en"]);let current=preferred_source_memory(Some(&remembered),None,&prefs).unwrap().unwrap();assert_eq!(current.audio_languages,vec!["es"]);
 let mut rows=vec![row("ready","addon:manual","CineStream",&["es"])];rows[0].quality_height=Some(720);
 let selected=reusable_source_context(&rows,Some(&remembered),None,&prefs,&RankingContext::default()).unwrap();assert!(selected.ready);assert!(!selected.fully_matches);
 assert!(!reusable_source_context(&[row("wrong","addon:manual","CineStream",&["en"])],Some(&remembered),None,&prefs,&RankingContext::default()).unwrap().ready);
 let suppliers=vec![("a".into(),"CineStream".into()),("b".into(),"Other".into())];assert_eq!(preferred_supplier_id(&remembered,&suppliers),Some("a".into()));let mut stable=remembered.clone();stable.source_id=Some("b".into());assert_eq!(preferred_supplier_id(&stable,&suppliers),Some("b".into()));stable.source_id=Some("removed".into());assert_eq!(preferred_supplier_id(&stable,&suppliers),None);
 assert_eq!(preferred_supplier_id(&remembered,&[("a".into(),"CineStream".into()),("b".into(),"cinestream".into())]),None);
 assert_eq!(preferred_source_decision(true,false,4999,false),PreferredSourceDecision::Start);assert_eq!(preferred_source_decision(true,false,5000,false),PreferredSourceDecision::Fallback);assert_eq!(preferred_source_decision(false,true,0,false),PreferredSourceDecision::Fallback);assert_eq!(preferred_source_decision(true,false,0,true),PreferredSourceDecision::Cancel);
}
