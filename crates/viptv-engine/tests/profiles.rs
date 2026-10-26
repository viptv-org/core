use viptv_engine::{profile::ProfileConfiguration, store::Store};

#[test]
fn local_profile_pin_avatar_and_delete_are_isolated() {
    let mut store = Store::memory().unwrap();
    let first = ProfileConfiguration::local("local".into(), "first".into(), "First".into());
    let second = ProfileConfiguration::local("local".into(), "second".into(), "Second".into());
    store.save_profile(&first, None).unwrap();
    store.save_profile(&second, None).unwrap();
    store.set_profile_pin(&first.scope, "", "1234").unwrap();
    assert!(store.has_profile_pin(&first.scope).unwrap());
    assert!(!store.has_profile_pin(&second.scope).unwrap());
    assert!(!store.verify_profile_pin(&first.scope, "9999").unwrap());
    assert!(store.verify_profile_pin(&first.scope, "1234").unwrap());
    assert!(
        store
            .set_profile_avatar(&first.scope, "zayne", "9999")
            .is_err()
    );
    store
        .set_profile_avatar(&first.scope, "zayne", "1234")
        .unwrap();
    assert_eq!(
        store
            .profile(&first.scope)
            .unwrap()
            .unwrap()
            .avatar_id
            .as_deref(),
        Some("zayne")
    );
    assert!(
        store
            .profile(&second.scope)
            .unwrap()
            .unwrap()
            .avatar_id
            .is_none()
    );
    assert!(
        store
            .rename_profile(&first.scope, "Renamed", "9999")
            .is_err()
    );
    store
        .rename_profile(&first.scope, "Renamed", "1234")
        .unwrap();
    assert_eq!(
        store.profile(&first.scope).unwrap().unwrap().name,
        "Renamed"
    );
    assert!(store.delete_profile(&first.scope, "9999").is_err());
    store.delete_profile(&first.scope, "1234").unwrap();
    assert!(store.profile(&first.scope).unwrap().is_none());
    assert!(!store.has_profile_pin(&first.scope).unwrap());
    assert!(store.profile(&second.scope).unwrap().is_some());
    assert!(store.delete_profile(&second.scope, "").is_err());
}

fn avatar(
    store: &Store,
    scope: &viptv_engine::profile::Scope,
) -> (Option<String>, String, Option<u16>) {
    let profile = store.profile(scope).unwrap().unwrap();
    (
        profile.avatar_id,
        profile.avatar_style,
        profile.avatar_choice,
    )
}

#[test]
fn local_profiles_default_to_the_backend_avatar() {
    let mut store = Store::memory().unwrap();
    let profile = ProfileConfiguration::local("local".into(), "p".into(), "P".into());
    store.save_profile(&profile, None).unwrap();
    assert_eq!(
        avatar(&store, &profile.scope),
        (None, "critters".into(), None)
    );
}

#[test]
fn catalog_avatar_choice_is_validated_saved_and_pin_gated() {
    let mut store = Store::memory().unwrap();
    let profile = ProfileConfiguration::local("local".into(), "p".into(), "P".into());
    store.save_profile(&profile, None).unwrap();
    store
        .set_profile_avatar_choice(&profile.scope, "notionists", 48, "")
        .unwrap();
    assert_eq!(
        avatar(&store, &profile.scope),
        (None, "notionists".into(), Some(48))
    );
    store
        .set_profile_avatar_choice(&profile.scope, "princesses", 10, "")
        .unwrap();
    for (style, choice) in [
        ("unknown", 1),
        ("", 1),
        ("critters", 0),
        ("critters", 49),
        ("princesses", 11),
        ("disney", 15),
    ] {
        assert!(
            store
                .set_profile_avatar_choice(&profile.scope, style, choice, "")
                .is_err(),
            "{style} {choice}"
        );
    }
    assert_eq!(
        avatar(&store, &profile.scope),
        (None, "princesses".into(), Some(10))
    );
    store.set_profile_pin(&profile.scope, "", "1234").unwrap();
    assert!(
        store
            .set_profile_avatar_choice(&profile.scope, "clay", 2, "9999")
            .is_err()
    );
    store
        .set_profile_avatar_choice(&profile.scope, "clay", 2, "1234")
        .unwrap();
    assert_eq!(
        avatar(&store, &profile.scope),
        (None, "clay".into(), Some(2))
    );
}

#[test]
fn legacy_avatar_ids_still_load_and_a_catalog_choice_takes_precedence() {
    let mut store = Store::memory().unwrap();
    let profile = ProfileConfiguration::local("local".into(), "p".into(), "P".into());
    store.save_profile(&profile, None).unwrap();
    store
        .set_profile_avatar(&profile.scope, "vynxc", "")
        .unwrap();
    assert_eq!(
        avatar(&store, &profile.scope),
        (Some("vynxc".into()), "critters".into(), None)
    );
    let saved = store.profile(&profile.scope).unwrap().unwrap();
    assert_eq!(saved.shown_avatar_id(), Some("vynxc"));
    assert!(
        store
            .set_profile_avatar(&profile.scope, "critters-1", "")
            .is_err()
    );

    // A catalog choice retires the legacy id.
    store
        .set_profile_avatar_choice(&profile.scope, "lorelei", 7, "")
        .unwrap();
    assert_eq!(
        avatar(&store, &profile.scope),
        (None, "lorelei".into(), Some(7))
    );
    // A later legacy write keeps the catalog choice, which stays in front.
    store
        .set_profile_avatar(&profile.scope, "zayne", "")
        .unwrap();
    let saved = store.profile(&profile.scope).unwrap().unwrap();
    assert_eq!(saved.avatar_choice, Some(7));
    assert_eq!(saved.shown_avatar_id(), None);

    // A profile row written before the catalog fields existed.
    let legacy: ProfileConfiguration = serde_json::from_str(
        r#"{"scope":{"connection_id":"local","account_id":"local","profile_id":"old","revision":0},"mode":"Local","name":"Old","avatar_id":"zayne","browsing_plugin":"stremio","assignments":[]}"#,
    )
    .unwrap();
    assert_eq!(legacy.shown_avatar_id(), Some("zayne"));
    assert_eq!(
        (legacy.avatar_style.as_str(), legacy.avatar_choice),
        ("critters", None)
    );
}

#[test]
fn avatar_worlds_match_the_design_catalog() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/avatar-catalog.json")).unwrap();
    let per_category = catalog["per_category"].as_u64().unwrap();
    let worlds: Vec<(String, u16)> = catalog["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|world| {
            let size = world["items"]
                .as_array()
                .map_or(per_category, |items| items.len() as u64);
            (world["style"].as_str().unwrap().to_owned(), size as u16)
        })
        .collect();
    let ours: Vec<(String, u16)> = viptv_engine::profile::AVATAR_WORLDS
        .iter()
        .map(|(style, size)| ((*style).to_owned(), *size))
        .collect();
    assert_eq!(ours, worlds);
    assert_eq!(
        ours.iter().map(|(_, size)| u64::from(*size)).sum::<u64>(),
        catalog["total"].as_u64().unwrap()
    );
}
