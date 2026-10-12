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
    let second: Value = serde_json::from_str(&engine.plan_home("second".into()).unwrap()).unwrap();
    assert!(second["requests"].as_array().unwrap().is_empty());
    drop(engine);
    let reopened = LocalEngine::new(path.to_string_lossy().into(), vault).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&reopened.addons().unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}
