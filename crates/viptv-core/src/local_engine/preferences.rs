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
