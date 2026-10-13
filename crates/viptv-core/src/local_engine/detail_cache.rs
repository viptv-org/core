//! Four scoped metadata bodies (at most 8 MiB) allow paging without repeat HTTP.
use super::*;
use viptv_engine::stremio;
#[derive(Clone)]
pub(super) struct Document {
    scope: Scope,
    created: std::time::Instant,
    provider: String,
    body: Arc<str>,
    recommendations: Vec<viptv_engine::presentation::Card>,
}
impl LocalEngine {
    pub(super) fn remember_detail_cards(
        &self,
        scope: &Scope,
        detail: &viptv_engine::presentation::DetailSnapshot,
    ) -> Result<(), CoreError> {
        let mut verified = self.verified_cards.lock().map_err(|_| CoreError::Bridge)?;
        verified.retain(|(profile, _, _), (cached, _)| {
            profile != &scope.profile_id || cached.revision == scope.revision
        });
        if verified.len() > 4096 {
            verified.clear();
        }
        for card in std::iter::once(&detail.item).chain(detail.episodes.iter()) {
            verified.insert(
                (
                    scope.profile_id.clone(),
                    card.identity.kind.clone(),
                    card.identity.id.clone(),
                ),
                (scope.clone(), card.clone()),
            );
        }
        Ok(())
    }
    pub(super) fn remember_detail_document(
        &self,
        scope: &Scope,
        kind: &str,
        id: &str,
        provider: &str,
        body: String,
        detail: &viptv_engine::presentation::DetailSnapshot,
    ) -> Result<(), CoreError> {
        if kind!="series" {return Ok(());}
        let mut documents = self
            .detail_documents
            .lock()
            .map_err(|_| CoreError::Bridge)?;
        documents.retain(|(profile, _, _), doc| {
            doc.created.elapsed() < std::time::Duration::from_secs(300)
                && (profile != &scope.profile_id || doc.scope == *scope)
        });
        let key = (scope.profile_id.clone(), kind.into(), id.into());
        if documents.len() >= 4 && !documents.contains_key(&key) {
            if let Some(oldest) = documents
                .iter()
                .min_by_key(|(_, doc)| doc.created)
                .map(|(key, _)| key.clone())
            {
                documents.remove(&oldest);
            }
        }
        documents.insert(
            key,
            Document {
                scope: scope.clone(),
                created: std::time::Instant::now(),
                provider: provider.into(),
                body: body.into(),
                recommendations: detail.recommendations.clone(),
            },
        );
        Ok(())
    }
}
#[uniffi::export]
impl LocalEngine {
    pub fn cached_detail(
        &self,
        profile: String,
        kind: String,
        id: String,
        offset: u32,
    ) -> Result<Option<String>, CoreError> {
        if id.is_empty()
            || id.len() > 512
            || !matches!(kind.as_str(), "movie" | "series")
            || offset > 100000
        {
            return Err(CoreError::InvalidInput);
        }
        let current: ProfileConfiguration =
            serde_json::from_str(&self.profile(profile.clone())?).map_err(|_| CoreError::Bridge)?;
        let snapshot = self
            .detail_snapshots
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .get(&(profile.clone(), kind.clone(), id.clone(), offset))
            .filter(|(scope, created, _)| {
                scope == &current.scope && created.elapsed() < std::time::Duration::from_secs(300)
            })
            .map(|(_, _, snapshot)| snapshot.clone());
        let snapshot = if let Some(snapshot) = snapshot {
            Some(snapshot)
        } else {
            let document = self
                .detail_documents
                .lock()
                .map_err(|_| CoreError::Bridge)?
                .get(&(profile.clone(), kind.clone(), id.clone()))
                .filter(|doc| {
                    doc.scope == current.scope
                        && doc.created.elapsed() < std::time::Duration::from_secs(300)
                })
                .cloned();
            if let Some(document) = document {
                let mut detail = stremio::detail(
                    &document.body,
                    &document.provider,
                    &kind,
                    &id,
                    offset as usize,
                )
                .map_err(|_| CoreError::InvalidInput)?;
                if detail.recommendations.is_empty() {
                    detail.recommendations = document.recommendations;
                }
                Some(detail)
            } else {
                None
            }
        };
        let after: ProfileConfiguration =
            serde_json::from_str(&self.profile(profile)?).map_err(|_| CoreError::Bridge)?;
        if after.scope != current.scope {
            return Ok(None);
        }
        if let Some(snapshot) = snapshot {
            self.remember_detail_cards(&current.scope, &snapshot)?;
            Ok(Some(
                serde_json::to_string(&snapshot).map_err(|_| CoreError::Bridge)?,
            ))
        } else {
            Ok(None)
        }
    }
}
