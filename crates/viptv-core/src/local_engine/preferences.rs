use super::*;
#[uniffi::export]
impl LocalEngine {
    pub fn profile_preferences(&self, profile: String) -> Result<String, CoreError> {
        let value = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .preferences(&local_scope(&profile))
            .map_err(|_| CoreError::InvalidInput)?;
        serde_json::to_string(&value).map_err(|_| CoreError::Bridge)
    }
    pub fn update_profile_preferences(
        &self,
        profile: String,
        revision: u64,
        preferences_json: String,
    ) -> Result<String, CoreError> {
        if preferences_json.len() > 4096 {
            return Err(CoreError::InvalidInput);
        }
        let preferences =
            serde_json::from_str(&preferences_json).map_err(|_| CoreError::InvalidInput)?;
        let value = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .update_preferences(&local_scope(&profile), revision, preferences)
            .map_err(|_| CoreError::InvalidInput)?;
        serde_json::to_string(&value).map_err(|_| CoreError::Bridge)
    }
}
#[uniffi::export]
impl LocalEngine {
    /// Supplied metadata is ignored; only IDs address current profile-owned private grants.
    pub fn rank_sources(&self, profile: String, sources_json: String) -> Result<String, CoreError> {
        if sources_json.len() > 128 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let input: Vec<serde_json::Value> =
            serde_json::from_str(&sources_json).map_err(|_| CoreError::InvalidInput)?;
        if input.len() > 160 {
            return Err(CoreError::InvalidInput);
        }
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let preferences = store
            .preferences(&selected.scope)
            .map_err(|_| CoreError::Bridge)?
            .preferences;
        drop(store);
        let grants = self.playable.lock().map_err(|_| CoreError::Bridge)?;
        let mut seen = std::collections::HashSet::new();
        let mut candidates = Vec::new();
        for value in input {
            let id = value
                .as_str()
                .or_else(|| value["id"].as_str())
                .ok_or(CoreError::InvalidInput)?;
            if !seen.insert(id.to_string()) {
                continue;
            }
            let grant = grants.get(id).ok_or(CoreError::InvalidInput)?;
            if grant.scope.profile_id != profile
                || grant.scope.revision != selected.scope.revision
                || grant
                    .expires
                    .is_some_and(|expiry| std::time::Instant::now() >= expiry)
            {
                return Err(CoreError::InvalidInput);
            }
            candidates.push(grant.candidate.clone());
        }
        viptv_engine::preferences::rank_sources(&mut candidates, &preferences);
        serde_json::to_string(&candidates).map_err(|_| CoreError::Bridge)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    struct Secrets(Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Secrets {
        fn get(&self, k: String) -> Result<Option<String>, CoreError> {
            Ok(self.0.lock().unwrap().get(&k).cloned())
        }
        fn put(&self, k: String, v: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().insert(k, v);
            Ok(())
        }
        fn remove(&self, k: String) -> Result<(), CoreError> {
            self.0.lock().unwrap().remove(&k);
            Ok(())
        }
    }
    #[test]
    fn native_ranking_uses_own_grants_and_persistent_preferences_instead_of_client_claims() {
        let dir = std::env::temp_dir().join(format!("viptv-rank-{}", uuid::Uuid::new_v4()));
        let engine = LocalEngine::new(
            dir.join("rank.sqlite").to_string_lossy().into(),
            Arc::new(Secrets(Mutex::new(Default::default()))),
        )
        .unwrap();
        engine
            .create_profile("other".into(), "Other".into())
            .unwrap();
        engine.install_addon("default".into(),"https://example.com/manifest.json".into(),r#"{"id":"sources","name":"Sources","version":"1","resources":["stream"],"types":["movie"]}"#.into()).unwrap();
        let plan: Value = serde_json::from_str(
            &engine
                .plan_sources("default".into(), "movie".into(), "tt1375666".into())
                .unwrap(),
        )
        .unwrap();
        let raw = json!({"streams":[{"name":"High","url":"https://media.example/high.mp4","quality_height":2160},{"name":"Compatible","url":"https://media.example/compatible.mp4","quality_height":1080},{"name":"Unknown","url":"https://media.example/unknown.mp4"}]});
        let sources: Value = serde_json::from_str(
            &engine
                .project_sources(
                    plan["operation"].as_str().unwrap().into(),
                    json!([{"provider":plan["requests"][0]["provider"],"body":raw.to_string()}])
                        .to_string(),
                )
                .unwrap(),
        )
        .unwrap();
        let mut preferences: Value =
            serde_json::from_str(&engine.profile_preferences("default".into()).unwrap()).unwrap();
        preferences["preferences"]["max_quality_height"] = json!(1080);
        engine
            .update_profile_preferences("default".into(), 0, preferences["preferences"].to_string())
            .unwrap();
        let mut supplied = sources.clone();
        supplied[0]["quality_height"] = json!(480);
        supplied[0]["label"] = json!("Spoofed best");
        let ranked: Value = serde_json::from_str(
            &engine
                .rank_sources("default".into(), supplied.to_string())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(ranked[0]["label"], "Compatible");
        assert_eq!(ranked[1]["label"], "Unknown");
        assert_eq!(ranked[2]["quality_height"], 2160);
        assert!(
            engine
                .rank_sources("other".into(), sources.to_string())
                .is_err()
        );
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
#[uniffi::export]
impl LocalEngine {
 pub fn bootstrap_profile(&self,profile:String)->Result<String,CoreError>{
  let home:serde_json::Value=serde_json::from_str(&self.home(profile.clone())?).map_err(|_|CoreError::Bridge)?;
  let library:serde_json::Value=serde_json::from_str(&self.library(profile.clone(),0,100)?).map_err(|_|CoreError::Bridge)?;
  let preferences:serde_json::Value=serde_json::from_str(&self.profile_preferences(profile)?).map_err(|_|CoreError::Bridge)?;
  Ok(serde_json::json!({"home":home,"library":library,"preferences":preferences}).to_string())
 }
}
