use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use viptv_engine::{
    capabilities::Capability,
    presentation::{HomeRow, HomeSnapshot, RowState},
    profile::{IntegrationConnection, Scope},
    stremio,
};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct StoredAddon {
    pub(super) manifest_url: String,
    pub(super) manifest: Value,
}

#[derive(Clone, Serialize)]
pub(super) struct CatalogRequest {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) provider_id: String,
    pub(super) url: Option<String>,
}

#[derive(Clone)]
pub(super) struct PendingHome {
    pub(super) scope: Scope,
    pub(super) rows: Vec<CatalogRequest>,
}

impl LocalEngine {
    fn key(&self, id: &str) -> String {
        format!("{}:{id}", self.namespace)
    }
    pub(super) fn addon(&self, id: &str) -> Result<StoredAddon, CoreError> {
        let raw = self.secrets.get(self.key(id))?.ok_or(CoreError::Bridge)?;
        serde_json::from_str(&raw).map_err(|_| CoreError::Bridge)
    }
}

#[cfg(test)]
mod tests;

#[uniffi::export]
impl LocalEngine {
    pub fn check_addon_url(&self, url: String) -> Result<String, CoreError> {
        stremio::manifest_url(&url).map_err(|_| CoreError::InvalidInput)
    }

    pub fn install_addon(
        &self,
        profile: String,
        url: String,
        raw_manifest: String,
    ) -> Result<String, CoreError> {
        let manifest_url = stremio::manifest_url(&url).map_err(|_| CoreError::InvalidInput)?;
        let manifest =
            stremio::parse_manifest(&raw_manifest).map_err(|_| CoreError::InvalidInput)?;
        let existing = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .connections("local")
            .map_err(|_| CoreError::Bridge)?;
        for connection in existing {
            if self
                .addon(&connection.id)
                .is_ok_and(|stored| stored.manifest_url == manifest_url)
            {
                self.store
                    .lock()
                    .map_err(|_| CoreError::Bridge)?
                    .assign(&profile, &connection)
                    .map_err(|_| CoreError::Bridge)?;
                return serde_json::to_string(&connection).map_err(|_| CoreError::Bridge);
            }
        }
        let id = format!("addon:{}", uuid::Uuid::new_v4());
        let record = IntegrationConnection {
            id: id.clone(),
            plugin_id: "stremio".into(),
            account_id: "local".into(),
            private_profile_id: None,
            name: manifest.name,
            enabled: true,
            revision: 0,
            capabilities: manifest.capabilities,
        };
        let secret = StoredAddon {
            manifest_url,
            manifest: serde_json::from_str(&raw_manifest).map_err(|_| CoreError::InvalidInput)?,
        };
        self.secrets.put(
            self.key(&id),
            serde_json::to_string(&secret).map_err(|_| CoreError::Bridge)?,
        )?;
        if self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .install(&profile, &record)
            .is_err()
        {
            let _ = self.secrets.remove(self.key(&id));
            return Err(CoreError::Bridge);
        }
        serde_json::to_string(&record).map_err(|_| CoreError::Bridge)
    }

    pub fn addons(&self) -> Result<String, CoreError> {
        serde_json::to_string(
            &self
                .store
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .connections("local")
                .map_err(|_| CoreError::Bridge)?,
        )
        .map_err(|_| CoreError::Bridge)
    }

    pub fn set_addon_enabled(
        &self,
        profile: String,
        id: String,
        enabled: bool,
    ) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .set_assignment(&profile, &id, enabled)
            .map_err(|_| CoreError::Bridge)
    }

    pub fn remove_addon(&self, id: String) -> Result<(), CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove_connection(&id)
            .map_err(|_| CoreError::Bridge)?;
        self.secrets.remove(self.key(&id))
    }

    pub fn plan_home(&self, profile_id: String) -> Result<String, CoreError> {
        let scope = local_scope(&profile_id);
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let profile = store
            .profile(&scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connections = store.connections("local").map_err(|_| CoreError::Bridge)?;
        let mut rows = Vec::new();
        for connection in connections
            .iter()
            .filter(|c| {
                c.enabled
                    && c.plugin_id == profile.browsing_plugin
                    && profile.assignments.iter().any(|a| {
                        a.connection_id == c.id && a.enabled.contains(&Capability::Discovery)
                    })
            })
            .take(8)
        {
            let secret = match self.addon(&connection.id) {
                Ok(s) => s,
                Err(_) => {
                    rows.push(CatalogRequest {
                        id: format!("{}:unavailable", connection.id),
                        title: connection.name.clone(),
                        provider_id: connection.id.clone(),
                        url: None,
                    });
                    continue;
                }
            };
            let parsed = stremio::parse_manifest(&secret.manifest.to_string())
                .map_err(|_| CoreError::Bridge)?;
            for catalog in parsed
                .catalogs
                .into_iter()
                .filter(|c| !c.required_extras)
                .take(4)
            {
                if rows.len() >= 12 {
                    break;
                }
                rows.push(CatalogRequest {
                    id: format!("{}:{}:{}", connection.id, catalog.kind, catalog.id),
                    title: format!("{} · {}", connection.name, catalog.name),
                    provider_id: connection.id.clone(),
                    url: stremio::catalog_url(&secret.manifest_url, &catalog).ok(),
                });
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        let pending = PendingHome {
            scope: profile.scope,
            rows: rows.clone(),
        };
        let mut operations = self.pending.lock().map_err(|_| CoreError::Bridge)?;
        operations.retain(|_, p| p.scope.profile_id != profile_id);
        operations.insert(id.clone(), pending);
        serde_json::to_string(&json!({"id":id,"requests":rows})).map_err(|_| CoreError::Bridge)
    }

    pub fn prepared_home(&self, operation: String) -> Result<String, CoreError> {
        let plan = self
            .pending
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .get(&operation)
            .cloned()
            .ok_or(CoreError::InvalidInput)?;
        let mut rows = self
            .continue_row(&plan.scope)?
            .into_iter()
            .collect::<Vec<_>>();
        rows.extend(plan.rows.iter().map(|request| {
            HomeRow {
                id: request.id.clone(),
                title: request.title.clone(),
                provider_id: request.provider_id.clone(),
                state: if request.url.is_some() {
                    RowState::Pending
                } else {
                    RowState::Failed
                },
                cards: vec![],
                next_cursor: None,
                message: request
                    .url
                    .is_none()
                    .then(|| "This addon is unavailable. Try again.".into()),
            }
        }));
        serde_json::to_string(&HomeSnapshot {
            scope: plan.scope.clone(),
            revision: plan.scope.revision,
            hero: None,
            rows,
            complete: false,
        })
        .map_err(|_| CoreError::Bridge)
    }

    pub fn project_home(&self, id: String, raw_results: String) -> Result<String, CoreError> {
        if raw_results.len() > 12 * 2 * 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let values: Value =
            serde_json::from_str(&raw_results).map_err(|_| CoreError::InvalidInput)?;
        let responses = values.as_array().ok_or(CoreError::InvalidInput)?;
        let pending = self
            .pending
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove(&id)
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
        let mut rows = pending
            .rows
            .iter()
            .map(|request| {
                let result = responses
                    .iter()
                    .find(|v| v["id"].as_str() == Some(&request.id));
                let cards = result
                    .and_then(|v| v["body"].as_str())
                    .and_then(|body| stremio::cards(body, &request.provider_id).ok());
                let state = if cards.is_none() {
                    RowState::Failed
                } else if cards.as_ref().is_some_and(Vec::is_empty) {
                    RowState::Empty
                } else {
                    RowState::Ready
                };
                HomeRow {
                    id: request.id.clone(),
                    title: request.title.clone(),
                    provider_id: request.provider_id.clone(),
                    state,
                    cards: cards.unwrap_or_default(),
                    next_cursor: None,
                    message: if request.url.is_none() || result.is_none() {
                        Some("This addon is unavailable. Try again.".into())
                    } else {
                        None
                    },
                }
            })
            .collect::<Vec<_>>();
        let hero = rows
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.landscape.is_some())
            .cloned();
        if let Some(queue) = self.continue_row(&pending.scope)? {
            rows.insert(0, queue);
        }
        serde_json::to_string(&HomeSnapshot {
            scope: pending.scope.clone(),
            revision: pending.scope.revision,
            hero,
            rows,
            complete: true,
        })
        .map_err(|_| CoreError::Bridge)
    }
}
