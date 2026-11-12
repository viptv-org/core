use crate::CoreError;
use std::sync::Mutex;
use viptv_engine::subtitle_sync::{CueInterval, Estimator, Observation};

#[cfg_attr(feature = "native", derive(uniffi::Object))]
pub struct SubtitleSyncBridge {
    inner: Mutex<Estimator>,
}

#[cfg_attr(feature = "native", uniffi::export)]
impl SubtitleSyncBridge {
    #[cfg_attr(feature = "native", uniffi::constructor)]
    pub fn new(epoch: u64, cues_json: String) -> Result<Self, CoreError> {
        let cues = parse_cues(&cues_json)?;
        let estimator = Estimator::new(epoch, cues).map_err(|_| CoreError::InvalidInput)?;
        Ok(Self {
            inner: Mutex::new(estimator),
        })
    }

    pub fn push(&self, epoch: u64, observations_json: String) -> Result<String, CoreError> {
        if observations_json.len() > 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let observations: Vec<Observation> =
            serde_json::from_str(&observations_json).map_err(|_| CoreError::InvalidInput)?;
        let mut estimator = self.inner.lock().map_err(|_| CoreError::Bridge)?;
        estimator
            .push(epoch, &observations)
            .map_err(|_| CoreError::InvalidInput)?;
        let estimate = estimator
            .estimate(epoch)
            .map_err(|_| CoreError::InvalidInput)?;
        serde_json::to_string(&estimate).map_err(|_| CoreError::Bridge)
    }
}

fn parse_cues(input: &str) -> Result<Vec<CueInterval>, CoreError> {
    if input.len() > 1024 * 1024 {
        return Err(CoreError::InvalidInput);
    }
    serde_json::from_str(input).map_err(|_| CoreError::InvalidInput)
}
