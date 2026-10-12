use crate::CoreError;
use std::{path::Path, sync::Mutex};
use viptv_engine::{
    profile::{ProfileConfiguration, Scope},
    store::Store,
};

#[derive(uniffi::Object)]
pub struct LocalEngine {
    store: Mutex<Store>,
}

fn local_scope(profile: &str) -> Scope {
    Scope {
        connection_id: "local".into(),
        account_id: "local".into(),
        profile_id: profile.into(),
        revision: 0,
    }
}

#[uniffi::export]
impl LocalEngine {
    #[uniffi::constructor]
    pub fn new(database_path: String) -> Result<Self, CoreError> {
        let path = Path::new(&database_path);
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|_| CoreError::Bridge)?;
        }
        let mut store = Store::open(path).map_err(|_| CoreError::Bridge)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|_| CoreError::Bridge)?;
        }
        if store
            .profile(&local_scope("default"))
            .map_err(|_| CoreError::Bridge)?
            .is_none()
        {
            let profile =
                ProfileConfiguration::local("local".into(), "default".into(), "Default".into());
            store
                .save_profile(&profile, None)
                .map_err(|_| CoreError::Bridge)?;
        }
        Ok(Self {
            store: Mutex::new(store),
        })
    }

    pub fn profiles(&self) -> Result<String, CoreError> {
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        serde_json::to_string(
            &store
                .profiles("local", "local")
                .map_err(|_| CoreError::Bridge)?,
        )
        .map_err(|_| CoreError::Bridge)
    }

    pub fn home(&self, id: String) -> Result<String, CoreError> {
        let profile: ProfileConfiguration=serde_json::from_str(&self.profile(id)?).map_err(|_|CoreError::Bridge)?;
        let snapshot=viptv_engine::presentation::HomeSnapshot {
            revision:profile.scope.revision,scope:profile.scope,hero:None,rows:vec![],complete:true,
        };
        serde_json::to_string(&snapshot).map_err(|_|CoreError::Bridge)
    }

    pub fn create_profile(&self, id: String, name: String) -> Result<String, CoreError> {
        if id.is_empty()
            || id.len() > 128
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || name.trim().is_empty()
            || name.chars().count() > 64
        {
            return Err(CoreError::InvalidInput);
        }
        let profile = ProfileConfiguration::local("local".into(), id, name.trim().into());
        let mut store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        if store
            .profiles("local", "local")
            .map_err(|_| CoreError::Bridge)?
            .len()
            >= 12
        {
            return Err(CoreError::InvalidInput);
        }
        store
            .save_profile(&profile, None)
            .map_err(|_| CoreError::Bridge)?;
        serde_json::to_string(&profile).map_err(|_| CoreError::Bridge)
    }

    pub fn profile(&self, id: String) -> Result<String, CoreError> {
        let profile = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .profile(&local_scope(&id))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        serde_json::to_string(&profile).map_err(|_| CoreError::Bridge)
    }
}
