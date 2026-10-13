use super::*;
use serde_json::{Value, json};
use viptv_engine::{
    capabilities::Capability,
    profile::Scope,
    stremio::{self, SourceCandidate},
};

pub(super) struct Request {
    provider: String,
    url: String,
}
pub(super) struct PendingSources {
    scope: Scope,
    kind: String,
    media_id: String,
    requests: Vec<Request>,
}
pub(super) struct Playable {
    pub(super) scope: Scope,
    pub(super) candidate:SourceCandidate,
    pub(super) kind: String,
    pub(super) media_id: String,
    pub(super) url: String,
    pub(super) headers: Vec<(String, String)>,
    pub(super) job_id: Option<String>,
    pub(super) expires: Option<std::time::Instant>,
}

#[uniffi::export]
impl LocalEngine {
    pub fn plan_sources(
        &self,
        profile: String,
        kind: String,
        id: String,
    ) -> Result<String, CoreError> {
        if id.len() > 512 || !["movie", "series"].contains(&kind.as_str()) {
            return Err(CoreError::InvalidInput);
        }
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let selected = store
            .profile(&local_scope(&profile))
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::InvalidInput)?;
        let connections = store.connections("local").map_err(|_| CoreError::Bridge)?;
        let mut requests = Vec::new();
        for connection in connections
            .iter()
            .filter(|c| {
                c.enabled
                    && selected.assignments.iter().any(|a| {
                        a.connection_id == c.id && a.enabled.contains(&Capability::Sources)
                    })
            })
            .take(12)
        {
            let Ok(secret) = self.addon(&connection.id) else {
                continue;
            };
            if let Some(url) =
                stremio::stream_url(&secret.manifest_url, &secret.manifest, &kind, &id)
            {
                requests.push(Request {
                    provider: connection.id.clone(),
                    url,
                });
            }
        }
        let operation = uuid::Uuid::new_v4().to_string();
        let wires = requests
            .iter()
            .map(|r| json!({"provider":r.provider,"url":r.url}))
            .collect::<Vec<_>>();
        let mut jobs = self.source_jobs.lock().map_err(|_| CoreError::Bridge)?;
        jobs.retain(|_, pending| pending.scope.profile_id != profile);
        jobs.insert(
            operation.clone(),
            PendingSources {
                scope: selected.scope,
                kind,
                media_id: id,
                requests,
            },
        );
        Ok(json!({"operation":operation,"requests":wires}).to_string())
    }

    pub fn project_sources(
        &self,
        operation: String,
        responses: String,
    ) -> Result<String, CoreError> {
        if responses.len() > 12 * 3 * 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let values: Value =
            serde_json::from_str(&responses).map_err(|_| CoreError::InvalidInput)?;
        let items = values.as_array().ok_or(CoreError::InvalidInput)?;
        let pending = self
            .source_jobs
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
        let mut playable = self.playable.lock().map_err(|_| CoreError::Bridge)?;
        playable.retain(|_, item| item.scope.profile_id != pending.scope.profile_id || item.job_id.is_some());
        let mut output = Vec::<SourceCandidate>::new();
        for request in &pending.requests {
            let raw = items
                .iter()
                .find(|v| v["provider"].as_str() == Some(&request.provider))
                .and_then(|v| v["body"].as_str());
            let sources = raw
                .and_then(|body| stremio::read_streams(body).ok())
                .unwrap_or_default();
            for source in sources {
                if output.len() >= 80 {
                    break;
                }
                let id = uuid::Uuid::new_v4().to_string();
                let candidate=SourceCandidate{id:id.clone(),provider_id:request.provider.clone(),label:source.label.clone(),detail:source.detail.clone(),quality_height:source.quality_height,audio_languages:source.audio_languages.clone()};
                playable.insert(
                    id.clone(),
                    Playable {
                        scope: pending.scope.clone(),
                        candidate:candidate.clone(),
                        kind: pending.kind.clone(),
                        media_id: pending.media_id.clone(),
                        url: source.url,
                        headers: source.headers,
                        expires: None,
                        job_id: None,
                    },
                );
                output.push(candidate);
            }
        }
        serde_json::to_string(&output).map_err(|_| CoreError::Bridge)
    }

    pub fn open_source(&self, id: String) -> Result<String, CoreError> {
        let play = self.playable.lock().map_err(|_| CoreError::Bridge)?;
        let source = play.get(&id).ok_or(CoreError::InvalidInput)?;
        if source.expires.is_some_and(|expiry| std::time::Instant::now() >= expiry) { return Err(CoreError::InvalidInput); }
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        let current = store
            .profile(&source.scope)
            .map_err(|_| CoreError::Bridge)?
            .ok_or(CoreError::Bridge)?;
        if current.scope.revision != source.scope.revision {
            return Err(CoreError::Bridge);
        }
        let resume = store
            .playback(&source.scope, &source.kind, &source.media_id)
            .map_err(|_| CoreError::Bridge)?
            .filter(|record| record.can_resume())
            .map(|record| record.position_millis)
            .unwrap_or(0);
        serde_json::to_string(&json!({"url":source.url,"headers":source.headers,"profile_id":source.scope.profile_id,"revision":source.scope.revision,"position_millis":resume}))
            .map_err(|_| CoreError::Bridge)
    }
}
