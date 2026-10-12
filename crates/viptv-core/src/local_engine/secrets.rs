use crate::CoreError;
use std::sync::Arc;

#[uniffi::export(with_foreign)]
pub trait SecretStore: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, CoreError>;
    fn put(&self, key: String, value: String) -> Result<(), CoreError>;
    fn remove(&self, key: String) -> Result<(), CoreError>;
}

#[cfg(not(target_os = "android"))]
struct SystemSecrets;

#[cfg(not(target_os = "android"))]
impl SecretStore for SystemSecrets {
    fn get(&self, key: String) -> Result<Option<String>, CoreError> {
        let entry =
            keyring::Entry::new("org.viptv.experiment", &key).map_err(|_| CoreError::Bridge)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(CoreError::Bridge),
        }
    }
    fn put(&self, key: String, value: String) -> Result<(), CoreError> {
        keyring::Entry::new("org.viptv.experiment", &key)
            .map_err(|_| CoreError::Bridge)?
            .set_password(&value)
            .map_err(|_| CoreError::Bridge)
    }
    fn remove(&self, key: String) -> Result<(), CoreError> {
        let result = keyring::Entry::new("org.viptv.experiment", &key)
            .map_err(|_| CoreError::Bridge)?
            .delete_credential();
        match result {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CoreError::Bridge),
        }
    }
}

#[cfg(not(target_os = "android"))]
#[uniffi::export]
pub fn system_secret_store() -> Arc<dyn SecretStore> {
    Arc::new(SystemSecrets)
}
