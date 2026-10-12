use super::*;
use serde_json::json;
use viptv_engine::{capabilities::Capability, stremio};

pub(super) struct PendingDetail {
    pub(super) scope: viptv_engine::profile::Scope,
    pub(super) kind: String,
    pub(super) id: String,
    pub(super) provider: String,
}

#[uniffi::export]
impl LocalEngine {
    pub fn plan_detail(
        &self,
        profile: String,
        kind: String,
        id: String,
        preferred: String,
    ) -> Result<String, CoreError> {
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connections = store.connections("local").map_err(|_| CoreError::Bridge)?;
        let mut eligible = connections
            .iter()
            .filter(|c| {
                c.enabled
                    && c.plugin_id == selected.browsing_plugin
                    && selected.assignments.iter().any(|a| {
                        a.connection_id == c.id && a.enabled.contains(&Capability::Metadata)
                    })
            })
            .collect::<Vec<_>>();
        eligible.sort_by_key(|c| if c.id == preferred { 0 } else { 1 });
        for connection in eligible {
            let Ok(stored) = self.addon(&connection.id) else {
                continue;
            };
            if let Some(url) = stremio::meta_url(&stored.manifest_url, &stored.manifest, &kind, &id)
            {
                let operation = uuid::Uuid::new_v4().to_string();
                let mut pending = self.details.lock().map_err(|_| CoreError::Bridge)?;
                pending.retain(|_, p| p.scope.profile_id != profile);
                pending.insert(
                    operation.clone(),
                    PendingDetail {
                        scope: selected.scope.clone(),
                        kind: kind.clone(),
                        id: id.clone(),
                        provider: connection.id.clone(),
                    },
                );
                return Ok(json!({"operation":operation,"url":url}).to_string());
            }
        }
        Err(CoreError::InvalidInput)
    }

    pub fn project_detail(
        &self,
        operation: String,
        body: String,
        offset: u32,
    ) -> Result<String, CoreError> {
        let pending = self
            .details
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove(&operation)
            .ok_or(CoreError::InvalidInput)?;
        let current = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .profile(&pending.scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::Bridge)?;
        if current.scope.revision != pending.scope.revision {
            return Err(CoreError::Bridge);
        }
        let detail = stremio::detail(
            &body,
            &pending.provider,
            &pending.kind,
            &pending.id,
            offset as usize,
        )
        .map_err(|_| CoreError::InvalidInput)?;
        serde_json::to_string(&detail).map_err(|_| CoreError::Bridge)
    }
}
