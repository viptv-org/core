use super::*;
use std::collections::HashMap;

#[derive(Default)]
struct MemorySecrets(Mutex<HashMap<String, String>>);
impl SecretStore for MemorySecrets {
    fn get(&self, key: String) -> Result<Option<String>, CoreError> {
        Ok(self.0.lock().unwrap().get(&key).cloned())
    }
    fn put(&self, key: String, value: String) -> Result<(), CoreError> {
        self.0.lock().unwrap().insert(key, value);
        Ok(())
    }
    fn remove(&self, key: String) -> Result<(), CoreError> {
        self.0.lock().unwrap().remove(&key);
        Ok(())
    }
}

#[test]
fn warm_home_retries_only_failed_rows_and_search_never_populates_home_cache() {
    let dir = std::env::temp_dir().join(format!("viptv-cache-{}", uuid::Uuid::new_v4()));
    let engine = LocalEngine::new(
        dir.join("local.sqlite").to_string_lossy().into(),
        Arc::new(MemorySecrets::default()),
    )
    .unwrap();
    engine.install_addon("default".into(), "https://example.com/manifest.json".into(),
        r#"{"id":"test","name":"Movies","version":"1","resources":["catalog"],"catalogs":[{"id":"first","type":"movie","extra":[{"name":"search"}]},{"id":"second","type":"movie"}]}"#.into()).unwrap();
    let plan: Value = serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    let first_id = plan["requests"][0]["id"].clone();
    let second_id = plan["requests"][1]["id"].clone();
    let results = json!([{"id":first_id,"body":r#"{"metas":[{"id":"tt1","type":"movie","name":"Home film"}]}"#}]);
    engine
        .project_home(plan["id"].as_str().unwrap().into(), results.to_string())
        .unwrap();
    let search: Value = serde_json::from_str(
        &engine
            .plan_search("default".into(), "Other".into())
            .unwrap(),
    )
    .unwrap();
    let results = json!([{"id":search["requests"][0]["id"],"body":r#"{"metas":[{"id":"tt2","type":"movie","name":"Search film"}]}"#}]);
    engine
        .project_home(search["id"].as_str().unwrap().into(), results.to_string())
        .unwrap();
    let warm: Value = serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    assert_eq!(warm["requests"].as_array().unwrap().len(), 1);
    assert_eq!(warm["requests"][0]["id"], second_id);
    let initial: Value = serde_json::from_str(
        &engine
            .prepared_home(warm["id"].as_str().unwrap().into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(initial["rows"][0]["cards"][0]["title"], "Home film");
    assert_eq!(initial["rows"][1]["state"], "Pending");
    let final_home: Value = serde_json::from_str(
        &engine
            .project_home(warm["id"].as_str().unwrap().into(), "[]".into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(final_home["rows"][0]["id"], first_id);
    assert_eq!(final_home["rows"][0]["state"], "Ready");
    assert_eq!(final_home["rows"][1]["state"], "Failed");
    assert!(final_home["rows"][1]["message"].is_string());
    drop(engine);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn subtitle_capabilities_are_profile_scoped_and_selections_expire_on_configuration_change() {
    let dir = std::env::temp_dir().join(format!("viptv-subtitles-{}", uuid::Uuid::new_v4()));
    let engine = LocalEngine::new(
        dir.join("local.sqlite").to_string_lossy().into(),
        Arc::new(MemorySecrets::default()),
    )
    .unwrap();
    engine
        .create_profile("second".into(), "Second".into())
        .unwrap();
    let installed: Value = serde_json::from_str(&engine.install_addon("default".into(), "https://addon.example/secret/manifest.json".into(),
        r#"{"id":"subs","name":"Subtitles","version":"1","resources":["subtitles"],"types":["movie"]}"#.into()).unwrap()).unwrap();
    let other: Value = serde_json::from_str(
        &engine
            .plan_subtitles("second".into(), "movie".into(), "tt1".into(), "{}".into())
            .unwrap(),
    )
    .unwrap();
    assert!(other["requests"].as_array().unwrap().is_empty());
    let plan: Value = serde_json::from_str(
        &engine
            .plan_subtitles("default".into(), "movie".into(), "tt1".into(), "{}".into())
            .unwrap(),
    )
    .unwrap();
    let responses = json!([{"provider":plan["requests"][0]["provider"],"body":r#"{"subtitles":[{"lang":"eng","url":"https://cdn.example/private-subtitle.srt"}]}"#}]);
    let snapshot: Value = serde_json::from_str(
        &engine
            .project_subtitles(
                plan["operation"].as_str().unwrap().into(),
                responses.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(snapshot["providers"][0]["state"], "Ready");
    assert_eq!(snapshot["candidates"][0]["language"], "eng");
    assert!(!snapshot.to_string().contains("private-subtitle"));
    let id = snapshot["candidates"][0]["id"].as_str().unwrap();
    let opened: Value = serde_json::from_str(&engine.open_subtitle(id.into()).unwrap()).unwrap();
    assert_eq!(opened["url"], "https://cdn.example/private-subtitle.srt");
    engine
        .set_addon_enabled(
            "default".into(),
            installed["id"].as_str().unwrap().into(),
            false,
        )
        .unwrap();
    assert!(engine.open_subtitle(id.into()).is_err());
    drop(engine);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn addon_secret_stays_out_of_sqlite_and_home_uses_its_catalog() {
    let dir = std::env::temp_dir().join(format!("viptv-local-{}", uuid::Uuid::new_v4()));
    let path = dir.join("local.sqlite");
    let vault = Arc::new(MemorySecrets::default());
    let engine = LocalEngine::new(path.to_string_lossy().into(), vault.clone()).unwrap();
    engine
        .create_profile("second".into(), "Second".into())
        .unwrap();
    let url = "https://example.com/secret-token/manifest.json";
    let manifest = r#"{"id":"test","name":"My movies","version":"1","resources":["catalog","meta","stream"],"catalogs":[{"id":"top","type":"movie","name":"Popular","extra":[{"name":"search"}]}]}"#;
    engine
        .install_addon("default".into(), url.into(), manifest.into())
        .unwrap();
    assert!(!engine.addons().unwrap().contains("secret-token"));
    assert!(
        !std::fs::read(&path)
            .unwrap()
            .windows(12)
            .any(|part| part == b"secret-token")
    );
    let plan: Value = serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    assert_eq!(plan["requests"].as_array().unwrap().len(), 1);
    let prepared: Value = serde_json::from_str(
        &engine
            .prepared_home(plan["id"].as_str().unwrap().into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prepared["rows"][0]["state"], "Pending");
    assert!(
        plan["requests"][0]["url"]
            .as_str()
            .unwrap()
            .contains("secret-token")
    );
    let response = json!([{"id":plan["requests"][0]["id"],"body":r#"{"metas":[{"id":"tt123","type":"movie","name":"Film","background":"https://images.example/landscape"}]}"#}]);
    let home: Value = serde_json::from_str(
        &engine
            .project_home(plan["id"].as_str().unwrap().into(), response.to_string())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(home["hero"]["title"], "Film");
    let warm: Value = serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    assert!(warm["requests"].as_array().unwrap().is_empty());
    let cached: Value = serde_json::from_str(
        &engine
            .prepared_home(warm["id"].as_str().unwrap().into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(cached["rows"][0]["cards"][0]["title"], "Film");
    assert_eq!(cached["hero"]["title"], "Film");
    assert_eq!(cached["complete"], true);
    let cached_final: Value = serde_json::from_str(
        &engine
            .project_home(warm["id"].as_str().unwrap().into(), "[]".into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(cached_final["rows"][0]["state"], "Ready");
    let search: Value = serde_json::from_str(
        &engine
            .plan_search("default".into(), "Two Words".into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(search["requests"].as_array().unwrap().len(), 1);
    assert!(
        search["requests"][0]["url"]
            .as_str()
            .unwrap()
            .contains("search=Two+Words")
    );
    let provider = home["rows"][0]["provider_id"].as_str().unwrap();
    let detail: Value = serde_json::from_str(
        &engine
            .plan_detail(
                "default".into(),
                "movie".into(),
                "tt123".into(),
                provider.into(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(
        detail["url"]
            .as_str()
            .unwrap()
            .ends_with("/meta/movie/tt123.json")
    );
    let item: Value = serde_json::from_str(
        &engine
            .project_detail(
                detail["operation"].as_str().unwrap().into(),
                r#"{"meta":{"id":"tt123","name":"Film"}}"#.into(),
                0,
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(item["item"]["title"], "Film");
    let sources: Value = serde_json::from_str(
        &engine
            .plan_sources("default".into(), "movie".into(), "tt123".into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(sources["requests"].as_array().unwrap().len(), 1);
    let response = json!([{"provider":sources["requests"][0]["provider"],"body":r#"{"streams":[{"name":"Full HD","url":"https://media.example/film.mp4","behaviorHints":{"proxyHeaders":{"request":{"Referer":"https://addon.example/"}}}}]}"#}]);
    let candidates: Value = serde_json::from_str(
        &engine
            .project_sources(
                sources["operation"].as_str().unwrap().into(),
                response.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(candidates.as_array().unwrap().len(), 1);
    assert!(!candidates.to_string().contains("media.example"));
    let prepared: Value = serde_json::from_str(
        &engine
            .open_source(candidates[0]["id"].as_str().unwrap().into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(prepared["url"], "https://media.example/film.mp4");
    assert_eq!(prepared["headers"][0][0], "Referer");
    engine
        .set_addon_enabled("default".into(), provider.into(), false)
        .unwrap();
    assert!(
        engine
            .open_source(candidates[0]["id"].as_str().unwrap().into())
            .is_err()
    );
    let disabled: Value =
        serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    assert!(disabled["requests"].as_array().unwrap().is_empty());
    engine
        .set_addon_enabled("default".into(), provider.into(), true)
        .unwrap();
    let second: Value = serde_json::from_str(&engine.plan_home("second".into()).unwrap()).unwrap();
    assert!(second["requests"].as_array().unwrap().is_empty());
    engine
        .install_addon("second".into(), url.into(), manifest.into())
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&engine.addons().unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let assigned: Value =
        serde_json::from_str(&engine.plan_home("second".into()).unwrap()).unwrap();
    assert_eq!(assigned["requests"].as_array().unwrap().len(), 1);
    // Assignment changes invalidate cached personalized catalogs, even when
    // they eventually restore the same enabled set.
    let invalidated: Value =
        serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
    assert_eq!(invalidated["requests"].as_array().unwrap().len(), 1);
    let cache_again = json!([{"id":invalidated["requests"][0]["id"],"body":r#"{"metas":[{"id":"tt123","type":"movie","name":"Film"}]}"#}]);
    engine
        .project_home(
            invalidated["id"].as_str().unwrap().into(),
            cache_again.to_string(),
        )
        .unwrap();
    drop(engine);
    let reopened = LocalEngine::new(path.to_string_lossy().into(), vault).unwrap();
    let restored: Value =
        serde_json::from_str(&reopened.plan_home("default".into()).unwrap()).unwrap();
    assert!(restored["requests"].as_array().unwrap().is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&reopened.addons().unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    reopened.remove_addon(provider.into()).unwrap();
    assert!(
        serde_json::from_str::<Value>(&reopened.addons().unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}
