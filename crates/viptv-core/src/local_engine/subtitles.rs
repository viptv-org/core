use super::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use viptv_engine::{
    capabilities::Capability,
    presentation::RowState,
    registry::Registry,
    stremio::{
        self, SubtitleCandidate, SubtitleExtras, SubtitleFile, SubtitleProvider, SubtitleSnapshot,
    },
};

struct Request {
    provider: String,
    name: String,
    url: Option<String>,
}
pub(super) struct PendingSubtitles {
    scope: Scope,
    requests: Vec<Request>,
}
pub(super) struct SelectedSubtitle {
    scope: Scope,
    file: SubtitleFile,
    created: Instant,
}

#[uniffi::export]
impl LocalEngine {
    /// Extras are optional actual file evidence; hosts must not invent hashes or sizes.
    pub fn plan_subtitles(
        &self,
        profile: String,
        kind: String,
        id: String,
        extras_json: String,
    ) -> Result<String, CoreError> {
        if extras_json.len() > 2048 {
            return Err(CoreError::InvalidInput);
        }
        let extras: SubtitleExtras =
            serde_json::from_str(&extras_json).map_err(|_| CoreError::InvalidInput)?;
        extras.validate().map_err(|_| CoreError::InvalidInput)?;
        if id.is_empty()
            || id.len() > 512
            || id.chars().any(char::is_control)
            || !["movie", "series"].contains(&kind.as_str())
        {
            return Err(CoreError::InvalidInput);
        }
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connections = store.connections("local").map_err(|_| CoreError::Bridge)?;
        let registry = Registry::builtin();
        let enabled = registry
            .connections_for(&selected, &connections, Capability::Subtitles)
            .map_err(|_| CoreError::Bridge)?;
        let mut requests = Vec::new();
        for connection in enabled
            .into_iter()
            .filter(|c| c.plugin_id == "stremio")
            .take(12)
        {
            let url = match self.addon(&connection.id) {
                Ok(addon) => match stremio::subtitle_url(
                    &addon.manifest_url,
                    &addon.manifest,
                    &kind,
                    &id,
                    &extras,
                )
                .map_err(|_| CoreError::InvalidInput)?
                {
                    Some(url) => Some(url),
                    None => continue,
                },
                Err(_) => None,
            };
            requests.push(Request {
                provider: connection.id.clone(),
                name: connection.name.clone(),
                url,
            });
        }
        let operation = uuid::Uuid::new_v4().to_string();
        let wire: Vec<_> = requests
            .iter()
            .map(|r| json!({"provider":r.provider,"url":r.url}))
            .collect();
        let mut jobs = self.subtitle_jobs.lock().map_err(|_| CoreError::Bridge)?;
        jobs.retain(|_, job| job.scope.profile_id != profile);
        jobs.insert(
            operation.clone(),
            PendingSubtitles {
                scope: selected.scope,
                requests,
            },
        );
        Ok(json!({"operation":operation,"requests":wire}).to_string())
    }

    pub fn project_subtitles(
        &self,
        operation: String,
        responses_json: String,
    ) -> Result<String, CoreError> {
        if responses_json.len() > 12 * 2 * 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let values: Value =
            serde_json::from_str(&responses_json).map_err(|_| CoreError::InvalidInput)?;
        let responses = values.as_array().ok_or(CoreError::InvalidInput)?;
        let pending = self
            .subtitle_jobs
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove(&operation)
            .ok_or(CoreError::InvalidInput)?;
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let current = store
            .profile(&pending.scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if current.scope.revision != pending.scope.revision {
            return Err(CoreError::Bridge);
        }
        drop(store);
        let mut selected = self
            .subtitle_selections
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        selected.retain(|_, selection| {
            selection.scope.profile_id != pending.scope.profile_id
                && selection.created.elapsed() < Duration::from_secs(600)
        });
        let mut snapshot = SubtitleSnapshot {
            scope: pending.scope.clone(),
            candidates: vec![],
            providers: vec![],
            complete: true,
        };
        for request in pending.requests {
            let result = responses
                .iter()
                .find(|v| v["provider"].as_str() == Some(&request.provider))
                .and_then(|v| v["body"].as_str())
                .and_then(|raw| stremio::read_subtitles(raw).ok());
            let state = match &result {
                None => RowState::Failed,
                Some(files) if files.is_empty() => RowState::Empty,
                _ => RowState::Ready,
            };
            snapshot.providers.push(SubtitleProvider {
                provider_id: request.provider.clone(),
                message: (state == RowState::Failed)
                    .then(|| "Subtitles from this addon are unavailable. Try again.".into()),
                state,
            });
            for file in result
                .unwrap_or_default()
                .into_iter()
                .take(80 - snapshot.candidates.len())
            {
                let id = uuid::Uuid::new_v4().to_string();
                snapshot.candidates.push(SubtitleCandidate {
                    id: id.clone(),
                    provider_id: request.provider.clone(),
                    language: file.language.clone(),
                    label: format!("{} · {}", file.language, request.name),
                });
                selected.insert(
                    id,
                    SelectedSubtitle {
                        scope: pending.scope.clone(),
                        file,
                        created: Instant::now(),
                    },
                );
            }
        }
        serde_json::to_string(&snapshot).map_err(|_| CoreError::Bridge)
    }

    pub fn open_subtitle(&self, candidate_id: String) -> Result<String, CoreError> {
        let selected = self
            .subtitle_selections
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        let selected = selected.get(&candidate_id).ok_or(CoreError::InvalidInput)?;
        if selected.created.elapsed() >= Duration::from_secs(600) {
            return Err(CoreError::InvalidInput);
        }
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let current = store
            .profile(&selected.scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        if current.scope.revision != selected.scope.revision {
            return Err(CoreError::Bridge);
        }
        Ok(json!({"url":selected.file.url,"language":selected.file.language,"profile_id":selected.scope.profile_id,"revision":selected.scope.revision}).to_string())
    }
}
