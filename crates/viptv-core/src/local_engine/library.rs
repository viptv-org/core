use super::*;
use viptv_engine::presentation::{Card, HomeRow, RowState};
impl LocalEngine {
    pub(super) fn home_revision(&self, scope: &Scope) -> Result<u64, CoreError> {
        Ok(scope.revision.saturating_add(
            self.store
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .library_revision(scope)
                .map_err(|_| CoreError::Bridge)?,
        ))
    }
    pub(super) fn favorites_row(&self, scope: &Scope) -> Result<Option<HomeRow>, CoreError> {
        let page = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .favorites(scope, 0, 12)
            .map_err(|_| CoreError::Bridge)?;
        if page.cards.is_empty() {
            return Ok(None);
        }
        Ok(Some(HomeRow {
            id: "favorites".into(),
            title: "My List".into(),
            provider_id: "library".into(),
            state: RowState::Ready,
            cards: page.cards,
            next_cursor: page.next_cursor,
            message: None,
        }))
    }
}
#[uniffi::export]
impl LocalEngine {
    pub fn library(&self, profile: String, offset: u32, limit: u32) -> Result<String, CoreError> {
        let page = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .favorites(&local_scope(&profile), offset, limit)
            .map_err(|_| CoreError::InvalidInput)?;
        serde_json::to_string(&page).map_err(|_| CoreError::Bridge)
    }
    pub fn save_favorite(
        &self,
        profile: String,
        revision: u64,
        card_json: String,
    ) -> Result<(), CoreError> {
        if card_json.len() > 64 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let card: Card = serde_json::from_str(&card_json).map_err(|_| CoreError::InvalidInput)?;
        let mut scope = local_scope(&profile);
        scope.revision = revision;
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .save_favorite(&scope, &card)
            .map_err(|_| CoreError::InvalidInput)
    }
    pub fn remove_favorite(
        &self,
        profile: String,
        revision: u64,
        kind: String,
        id: String,
    ) -> Result<(), CoreError> {
        let mut scope = local_scope(&profile);
        scope.revision = revision;
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove_favorite(&scope, &kind, &id)
            .map_err(|_| CoreError::InvalidInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[derive(Default)]
    struct Secrets(Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Secrets {
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
    fn local_library_bridge_round_trips_my_list_across_restart_and_prepared_home() {
        let dir =
            std::env::temp_dir().join(format!("viptv-local-library-{}", uuid::Uuid::new_v4()));
        let path = dir.join("library.sqlite").to_string_lossy().to_string();
        let secrets = Arc::new(Secrets::default());
        let engine = LocalEngine::new(path.clone(), secrets.clone()).unwrap();
        let mut card = json!({"identity":{"id":"opaque-film","kind":"movie","ids":[{"namespace":"imdb","value":"tt1234567"}],"episode":null,"edition":null},"provider_id":"stremio","title":"My film","subtitle":null,"description":null,"poster":null,"landscape":"https://images.example/frame.jpg","logo":null,"progress":null,"badges":[],"parent_identity":null});
        engine
            .save_favorite("default".into(), 0, card.to_string())
            .unwrap();
        let page: Value =
            serde_json::from_str(&engine.library("default".into(), 0, 40).unwrap()).unwrap();
        assert!(page["cards"][0]["added_at_millis"].as_u64().is_some());
        card["added_at_millis"]=page["cards"][0]["added_at_millis"].clone();
        assert_eq!(page["cards"][0], card);
        let home: Value = serde_json::from_str(&engine.home("default".into()).unwrap()).unwrap();
        assert_eq!(home["rows"][0]["id"], "favorites");
        assert_eq!(home["revision"], 1);
        let plan: Value =
            serde_json::from_str(&engine.plan_home("default".into()).unwrap()).unwrap();
        let prepared: Value = serde_json::from_str(
            &engine
                .prepared_home(plan["id"].as_str().unwrap().into())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(prepared["rows"][0]["cards"][0], card);
        let projected: Value = serde_json::from_str(
            &engine
                .project_home(plan["id"].as_str().unwrap().into(), "[]".into())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(projected["rows"][0]["cards"][0], card);
        assert!(
            engine
                .remove_favorite("default".into(), 9, "movie".into(), "opaque-film".into())
                .is_err()
        );
        drop(engine);
        let engine = LocalEngine::new(path, secrets).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&engine.library("default".into(), 0, 40).unwrap())
                .unwrap()["cards"][0],
            card
        );
        engine
            .remove_favorite("default".into(), 0, "movie".into(), "opaque-film".into())
            .unwrap();
        assert!(
            serde_json::from_str::<Value>(&engine.library("default".into(), 0, 40).unwrap())
                .unwrap()["cards"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
