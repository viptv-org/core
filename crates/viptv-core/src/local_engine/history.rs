use super::*;
use std::time::{SystemTime, UNIX_EPOCH};
use viptv_engine::{
    playback::HistoryEntry,
    presentation::{Card, HomeRow, RowState},
    profile::Scope,
};

impl LocalEngine {
    pub(super) fn continue_row(&self, scope: &Scope) -> Result<Option<HomeRow>, CoreError> {
        let records = self
            .store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .history(scope, 100)
            .map_err(|_| CoreError::Bridge)?;
        let mut cards = records
            .iter()
            .filter(|r| r.can_resume())
            .take(12)
            .map(|r| r.card())
            .collect::<Vec<_>>();
        self.merge_cached_presentation(scope, &mut cards)?;
        if cards.is_empty() {
            return Ok(None);
        }
        Ok(Some(HomeRow {browse:Some(viptv_engine::home_browse::HomeBrowseAction::continuation()),
            id: "continue".into(),
            title: "Continue Watching".into(),
            provider_id: "library".into(),
            state: RowState::Ready,
            cards,
            next_cursor: None,
            message: None,
        }))
    }
}

#[uniffi::export]
impl LocalEngine {
    pub fn record_progress(
        &self,
        profile: String,
        revision: u64,
        card_json: String,
        position_millis: u64,
        duration_millis: u64,
    ) -> Result<(), CoreError> {
        if card_json.len() > 32 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let card: Card = serde_json::from_str(&card_json).map_err(|_| CoreError::InvalidInput)?;
        let scope = Scope {
            connection_id: "local".into(),
            account_id: "local".into(),
            profile_id: profile,
            revision,
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CoreError::Bridge)?
            .as_millis() as u64;
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .record_playback(&scope, &card, position_millis, duration_millis, now)
            .map_err(|_| CoreError::Bridge)
    }

    pub fn history(&self, profile: String) -> Result<String, CoreError> {
        let scope = local_scope(&profile);
        let store = self.store.lock().map_err(|_| CoreError::Bridge)?;
        if store
            .profile(&scope)
            .map_err(|_| CoreError::Bridge)?
            .is_none()
        {
            return Err(CoreError::InvalidInput);
        }
        let records = store.history(&scope, 100).map_err(|_| CoreError::Bridge)?;
        serde_json::to_string(&records.iter().map(HistoryEntry::from).collect::<Vec<_>>())
            .map_err(|_| CoreError::Bridge)
    }

    pub fn clear_history(&self, profile: String) -> Result<(), CoreError> {
        let scope = local_scope(&profile);
        self.store
            .lock()
            .map_err(|_| CoreError::Bridge)?
            .clear_history(&scope)
            .map_err(|_| CoreError::Bridge)
    }
}
