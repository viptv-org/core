use crate::CoreError;
use serde::Serialize;
use std::sync::{Mutex, TryLockError};
use viptv_engine::subtitle_sync::{CueInterval, Estimate, Estimator};

pub const MAX_PCM_BYTES: usize = 32 * 1024;
#[cfg(feature = "subtitle-vad")]
const VAD_PATCH: &str = viptv_vad::PATCH_VERSION;
#[cfg(not(feature = "subtitle-vad"))]
const VAD_PATCH: &str = "unavailable";
const VAD_REVISION: &str = "e283ca41df3a84b0e87fb1f5cb9b21580a286b09";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PcmStatus {
    Ok,
    Unavailable,
    Busy,
    StaleEpoch,
    UnsupportedFormat,
    UnsupportedRate,
    UnsupportedChannels,
    InvalidBuffer,
    InputTooLarge,
    NeedsNewEpoch,
    InvalidTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PcmResult {
    epoch: u64,
    status: PcmStatus,
    processed_frames: u64,
    emitted_frames: u32,
    pending_samples: usize,
    gaps: u64,
    last_frame_start_us: Option<i64>,
    last_frame_end_us: Option<i64>,
    estimate: Option<Estimate>,
}

impl PcmResult {
    fn empty(epoch: u64, status: PcmStatus) -> Self {
        Self {
            epoch,
            status,
            processed_frames: 0,
            emitted_frames: 0,
            pending_samples: 0,
            gaps: 0,
            last_frame_start_us: None,
            last_frame_end_us: None,
            estimate: None,
        }
    }
}

enum Session {
    #[cfg(feature = "subtitle-vad")]
    Active(active::PcmSession),
    #[cfg(not(feature = "subtitle-vad"))]
    Disabled,
}

#[cfg_attr(feature = "native", derive(uniffi::Object))]
pub struct SubtitlePcmSyncBridge {
    epoch: u64,
    inner: Mutex<Session>,
}

#[cfg_attr(feature = "native", uniffi::export)]
pub fn subtitle_pcm_capabilities() -> String {
    let available = cfg!(feature = "subtitle-vad");
    serde_json::json!({"available":available, "encoding":"pcmS16Le", "frameMs":20,
        "sampleRates":if available { vec![8_000,16_000,32_000,48_000] } else { vec![] },
        "channels":if available { vec![1,2] } else { vec![] }, "maxInputBytes":MAX_PCM_BYTES,
        "vadRevision":VAD_REVISION,"vadMode":2,"frontendVersion":"pcm16-webrtc20-v2","vadPatch":VAD_PATCH,"workerOnly":true})
    .to_string()
}

#[cfg_attr(feature = "native", uniffi::export)]
impl SubtitlePcmSyncBridge {
    #[cfg_attr(feature = "native", uniffi::constructor)]
    pub fn new(epoch: u64, cues_json: String) -> Result<Self, CoreError> {
        if cues_json.len() > 1024 * 1024 {
            return Err(CoreError::InvalidInput);
        }
        let cues: Vec<CueInterval> =
            serde_json::from_str(&cues_json).map_err(|_| CoreError::InvalidInput)?;
        let estimator = Estimator::new(epoch, cues).map_err(|_| CoreError::InvalidInput)?;
        #[cfg(feature = "subtitle-vad")]
        let session = Session::Active(active::PcmSession::new(epoch, estimator)?);
        #[cfg(not(feature = "subtitle-vad"))]
        let session = {
            drop(estimator);
            Session::Disabled
        };
        Ok(Self {
            epoch,
            inner: Mutex::new(session),
        })
    }

    pub fn push_pcm16(
        &self,
        epoch: u64,
        start_us: i64,
        sample_rate: u32,
        channels: u8,
        encoding: String,
        pcm_bytes: Vec<u8>,
    ) -> Result<String, CoreError> {
        if epoch != self.epoch {
            return encode(PcmResult::empty(self.epoch, PcmStatus::StaleEpoch));
        }
        let mut session = match self.inner.try_lock() {
            Ok(session) => session,
            Err(TryLockError::WouldBlock) => {
                return encode(PcmResult::empty(epoch, PcmStatus::Busy));
            }
            Err(TryLockError::Poisoned(_)) => return Err(CoreError::Bridge),
        };
        match &mut *session {
            #[cfg(feature = "subtitle-vad")]
            Session::Active(active) => {
                encode(active.push(start_us, sample_rate, channels, &encoding, &pcm_bytes)?)
            }
            #[cfg(not(feature = "subtitle-vad"))]
            Session::Disabled => {
                let _ = (start_us, sample_rate, channels, encoding, pcm_bytes);
                encode(PcmResult::empty(epoch, PcmStatus::Unavailable))
            }
        }
    }

    pub fn gap(&self, epoch: u64, start_us: i64, end_us: i64) -> Result<String, CoreError> {
        if epoch != self.epoch {
            return encode(PcmResult::empty(self.epoch, PcmStatus::StaleEpoch));
        }
        let mut session = match self.inner.try_lock() {
            Ok(session) => session,
            Err(TryLockError::WouldBlock) => {
                return encode(PcmResult::empty(epoch, PcmStatus::Busy));
            }
            Err(TryLockError::Poisoned(_)) => return Err(CoreError::Bridge),
        };
        match &mut *session {
            #[cfg(feature = "subtitle-vad")]
            Session::Active(active) => encode(active.gap(start_us, end_us)?),
            #[cfg(not(feature = "subtitle-vad"))]
            Session::Disabled => {
                let _ = (start_us, end_us);
                encode(PcmResult::empty(epoch, PcmStatus::Unavailable))
            }
        }
    }
}

fn encode(result: PcmResult) -> Result<String, CoreError> {
    serde_json::to_string(&result).map_err(|_| CoreError::Bridge)
}

#[cfg(feature = "subtitle-vad")]
mod active;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, mpsc},
        time::Duration,
    };

    #[test]
    fn concurrent_calls_return_busy_instead_of_waiting_for_worker() {
        let bridge = Arc::new(SubtitlePcmSyncBridge::new(1, "[]".into()).unwrap());
        let guard = bridge.inner.lock().unwrap();
        let worker_bridge = bridge.clone();
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            send.send(worker_bridge.push_pcm16(1, 0, 48000, 2, "pcmS16Le".into(), vec![0; 3840]))
                .unwrap();
        });
        let result = receive.recv_timeout(Duration::from_secs(1));
        drop(guard);
        worker.join().unwrap();
        let result: serde_json::Value = serde_json::from_str(&result.unwrap().unwrap()).unwrap();
        assert_eq!(result["status"], "busy");
    }
}
