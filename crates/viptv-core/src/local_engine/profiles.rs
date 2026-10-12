use super::*;
use serde::Serialize;

#[derive(Serialize)]
struct ProfileCard {
    id: String,
    name: String,
    locked: bool,
    avatar_id: Option<String>,
}

#[uniffi::export]
impl LocalEngine {
    pub fn profile_cards(&self) -> Result<String, CoreError> {
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let cards = store
            .profiles("local", "local")
            .map_err(|_| CoreError::Bridge)?
            .into_iter()
            .map(|profile| {
                Ok(ProfileCard {
                    locked: store
                        .has_profile_pin(&profile.scope)
                        .map_err(|_| CoreError::Bridge)?,
                    id: profile.scope.profile_id,
                    name: profile.name,
                    avatar_id: profile.avatar_id,
                })
            })
            .collect::<Result<Vec<_>, CoreError>>()?;
        serde_json::to_string(&cards).map_err(|_| CoreError::Bridge)
    }

    pub fn set_profile_avatar(
        &self,
        id: String,
        avatar_id: String,
        pin: String,
    ) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .set_profile_avatar(&local_scope(&id), &avatar_id, &pin)
            .map_err(|_| CoreError::InvalidInput)
    }

    pub fn rename_profile(&self, id: String, name: String, pin: String) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .rename_profile(&local_scope(&id), &name, &pin)
            .map_err(|_| CoreError::InvalidInput)
    }

    pub fn delete_profile(&self, id: String, pin: String) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .delete_profile(&local_scope(&id), &pin)
            .map_err(|_| CoreError::InvalidInput)
    }

    pub fn set_profile_pin(
        &self,
        id: String,
        old_pin: String,
        new_pin: String,
    ) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .set_profile_pin(&local_scope(&id), &old_pin, &new_pin)
            .map_err(|_| CoreError::InvalidInput)
    }

    pub fn verify_profile_pin(&self, id: String, pin: String) -> Result<bool, CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .verify_profile_pin(&local_scope(&id), &pin)
            .map_err(|_| CoreError::Bridge)
    }
}
