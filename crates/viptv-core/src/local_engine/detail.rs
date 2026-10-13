use super::*;
use serde_json::{Value, json};
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
                if pending.len() >= 64 {
                    return Err(CoreError::Bridge);
                }
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
        let mut detail = stremio::detail(
            &body,
            &pending.provider,
            &pending.kind,
            &pending.id,
            offset as usize,
        )
        .map_err(|_| CoreError::InvalidInput)?;
        if detail.recommendations.is_empty() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |t| t.as_secs() as i64);
            detail.recommendations = self
                .store
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .related_catalog_cards(&pending.scope, &detail.item, now)
                .unwrap_or_default();
        }
        self.remember_detail_cards(&pending.scope,&detail)?;
        self.remember_detail_document(&pending.scope,&pending.kind,&pending.id,&pending.provider,body,&detail)?;
        let mut snapshots = self
            .detail_snapshots
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        snapshots.retain(|_, (scope, created, _)| {
            created.elapsed() < std::time::Duration::from_secs(300)
                && (scope.profile_id != pending.scope.profile_id
                    || scope.revision == pending.scope.revision)
        });
        if snapshots.len() >= 128 {
            snapshots.clear();
        }
        snapshots.insert(
            (
                pending.scope.profile_id.clone(),
                pending.kind,
                pending.id,
                offset,
            ),
            (pending.scope, std::time::Instant::now(), detail.clone()),
        );
        serde_json::to_string(&detail).map_err(|_| CoreError::Bridge)
    }
}

pub(super) struct PendingEnrichment {
    scope: viptv_engine::profile::Scope,
    snapshot: viptv_engine::presentation::HomeSnapshot,
    requests: Vec<String>,
}
fn enrich_snapshot(
    snapshot: &mut viptv_engine::presentation::HomeSnapshot,
    cards: &[viptv_engine::presentation::Card],
) {
    for original in snapshot
        .hero
        .iter_mut()
        .chain(snapshot.featured.iter_mut())
        .chain(snapshot.rows.iter_mut().flat_map(|r| r.cards.iter_mut()))
    {
        let identity = original
            .parent_identity
            .as_ref()
            .unwrap_or(&original.identity);
        if let Some(metadata) = cards
            .iter()
            .find(|c| c.identity.id == identity.id && c.identity.kind == identity.kind)
        {
            *original = viptv_engine::presentation::enrich_card(original, metadata);
        }
    }
}
#[uniffi::export]
impl LocalEngine {
    pub fn plan_home_enrichment(
        &self,
        profile: String,
        home_json: String,
    ) -> Result<String, CoreError> {
        if home_json.len() > 2 * 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let mut snapshot: viptv_engine::presentation::HomeSnapshot =
            serde_json::from_str(&home_json).map_err(|_| CoreError::InvalidInput)?;
        let current: ProfileConfiguration =
            serde_json::from_str(&self.profile(profile.clone())?).map_err(|_| CoreError::Bridge)?;
        if snapshot.scope != current.scope
            || snapshot.featured.len() > 5
            || snapshot.rows.len() > 14
            || snapshot.rows.iter().any(|r| r.cards.len() > 24)
        {
            return Err(CoreError::InvalidInput);
        }
        if snapshot.featured.is_empty() {
            snapshot.featured = snapshot.hero.iter().cloned().collect();
        }
        let candidates = snapshot
            .featured
            .iter()
            .chain(
                snapshot
                    .rows
                    .iter()
                    .filter(|r| r.id == "continue")
                    .flat_map(|r| r.cards.iter().take(12)),
            )
            .cloned()
            .collect::<Vec<_>>();
        let verified = self.verified_cards.lock().map_err(|_| CoreError::Bridge)?;
        let mut cached = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut needed = Vec::new();
        for card in candidates {
            let identity = card.parent_identity.as_ref().unwrap_or(&card.identity);
            if !seen.insert((identity.kind.clone(), identity.id.clone())) {
                continue;
            }
            if let Some((scope, metadata)) = verified
                .get(&(profile.clone(), identity.kind.clone(), identity.id.clone()))
                .filter(|(scope, _)| scope == &current.scope)
            {
                let _ = scope;
                cached.push(metadata.clone());
            } else {
                needed.push((identity.clone(), card.provider_id));
            }
        }
        drop(verified);
        enrich_snapshot(&mut snapshot, &cached);
        let mut retired = Vec::new();
        self.enrichments
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .retain(|_, pending| {
                if pending.scope.profile_id == profile {
                    retired.extend(pending.requests.iter().cloned());
                    false
                } else {
                    true
                }
            });
        {
            let mut pending = self.details.lock().map_err(|_| CoreError::Bridge)?;
            for id in retired {
                pending.remove(&id);
            }
        }
        let mut staged = Vec::new();
        let mut requests = Vec::new();
        let mut operations = Vec::new();
        for (identity, provider) in needed.into_iter().take(17) {
            if let Ok(plan) =
                self.plan_detail(profile.clone(), identity.kind, identity.id, provider)
            {
                let request: Value = serde_json::from_str(&plan).map_err(|_| CoreError::Bridge)?;
                if let Some(operation) = request["operation"].as_str() {
                    if let Some(pending) = self
                        .details
                        .lock()
                        .map_err(|_| CoreError::Bridge)?
                        .remove(operation)
                    {
                        staged.push((operation.to_owned(), pending));
                    }
                    operations.push(operation.to_owned());
                    requests.push(request);
                }
            }
        }
        self.details
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .extend(staged);
        let operation = uuid::Uuid::new_v4().to_string();
        let prepared = snapshot.clone();
        let mut pending = self.enrichments.lock().map_err(|_| CoreError::Bridge)?;
        pending.retain(|_, p| p.scope.profile_id != profile);
        if pending.len() >= 12 {
            return Err(CoreError::Bridge);
        }
        pending.insert(
            operation.clone(),
            PendingEnrichment {
                scope: current.scope,
                snapshot,
                requests: operations,
            },
        );
        Ok(json!({"operation":operation,"requests":requests,"prepared":prepared}).to_string())
    }
    pub fn project_home_enrichment(
        &self,
        operation: String,
        raw_results: String,
    ) -> Result<String, CoreError> {
        if raw_results.len() > 34 * 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let values: Value =
            serde_json::from_str(&raw_results).map_err(|_| CoreError::InvalidInput)?;
        let results = values.as_array().ok_or(CoreError::InvalidInput)?;
        let mut pending = self
            .enrichments
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .remove(&operation)
            .ok_or(CoreError::InvalidInput)?;
        let current: ProfileConfiguration =
            serde_json::from_str(&self.profile(pending.scope.profile_id.clone())?)
                .map_err(|_| CoreError::Bridge)?;
        if current.scope != pending.scope {
            return Err(CoreError::Bridge);
        }
        let mut cards = Vec::new();
        for request in &pending.requests {
            if let Some(body) = results
                .iter()
                .find(|r| r["operation"].as_str() == Some(request))
                .and_then(|r| r["body"].as_str())
            {
                if let Ok(detail) = self.project_detail(request.clone(), body.into(), 0) {
                    if let Ok(detail) =
                        serde_json::from_str::<viptv_engine::presentation::DetailSnapshot>(&detail)
                    {
                        cards.push(detail.item);
                    }
                }
            }
            self.details
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .remove(request);
        }
        enrich_snapshot(&mut pending.snapshot, &cards);
        let current: ProfileConfiguration =
            serde_json::from_str(&self.profile(pending.scope.profile_id.clone())?)
                .map_err(|_| CoreError::Bridge)?;
        if current.scope != pending.scope {
            return Err(CoreError::Bridge);
        }
        serde_json::to_string(&pending.snapshot).map_err(|_| CoreError::Bridge)
    }
}

impl LocalEngine {
    pub(super) fn merge_cached_presentation(
        &self,
        scope: &Scope,
        cards: &mut [viptv_engine::presentation::Card],
    ) -> Result<(), CoreError> {
        let verified = self.verified_cards.lock().map_err(|_| CoreError::Bridge)?;
        for card in cards {
            let identity = card.parent_identity.as_ref().unwrap_or(&card.identity);
            if let Some((cached_scope, metadata)) = verified
                .get(&(
                    scope.profile_id.clone(),
                    identity.kind.clone(),
                    identity.id.clone(),
                ))
                .filter(|(cached_scope, _)| cached_scope == scope)
            {
                let _ = cached_scope;
                *card = viptv_engine::presentation::enrich_card(card, metadata);
            }
        }
        Ok(())
    }
}
