use super::{CoreError, Estimate, Estimator, MAX_PCM_BYTES, PcmResult, PcmStatus};
use viptv_engine::subtitle_sync::{Activity, Observation};
use viptv_vad::{SAMPLE_RATES, Vad};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Format {
    rate: u32,
    channels: u8,
}

pub(super) struct PcmSession {
    epoch: u64,
    estimator: Estimator,
    vad: Vad,
    format: Option<Format>,
    base_us: i64,
    input_frames: u64,
    partial: [i16; 960],
    partial_len: usize,
    frame_start_us: i64,
    last_observation_end: Option<i64>,
    processed_frames: u64,
    gaps: u64,
    last_frame: Option<(i64, i64)>,
}

impl PcmSession {
    pub(super) fn new(epoch: u64, estimator: Estimator) -> Result<Self, CoreError> {
        Ok(Self {
            epoch,
            estimator,
            vad: Vad::new().map_err(|_| CoreError::Bridge)?,
            format: None,
            base_us: 0,
            input_frames: 0,
            partial: [0; 960],
            partial_len: 0,
            frame_start_us: 0,
            last_observation_end: None,
            processed_frames: 0,
            gaps: 0,
            last_frame: None,
        })
    }

    pub(super) fn push(
        &mut self,
        start_us: i64,
        rate: u32,
        channels: u8,
        encoding: &str,
        pcm: &[u8],
    ) -> Result<PcmResult, CoreError> {
        if !valid_time(start_us) {
            return self.result(PcmStatus::InvalidTime, 0);
        }
        if self.last_observation_end.is_some_and(|end| start_us < end) {
            return self.result(PcmStatus::NeedsNewEpoch, 0);
        }
        let error = if pcm.len() > MAX_PCM_BYTES {
            Some(PcmStatus::InputTooLarge)
        } else if encoding != "pcmS16Le" {
            Some(PcmStatus::UnsupportedFormat)
        } else if !SAMPLE_RATES.contains(&rate) {
            Some(PcmStatus::UnsupportedRate)
        } else if ![1, 2].contains(&channels) {
            Some(PcmStatus::UnsupportedChannels)
        } else if pcm.is_empty() || pcm.len() % (2 * channels as usize) != 0 {
            Some(PcmStatus::InvalidBuffer)
        } else {
            None
        };
        if let Some(status) = error {
            self.restart_at(start_us, true)?;
            return self.result(status, 0);
        }
        let frames = pcm.len() / (2 * channels as usize);
        let format = Format { rate, channels };
        if self.format == Some(format) {
            let expected = i128::from(self.base_us)
                + i128::from(self.input_frames) * 1_000_000 / i128::from(rate);
            let delta = i128::from(start_us) - expected;
            if delta < -1 {
                return self.result(PcmStatus::NeedsNewEpoch, 0);
            }
            if delta > 1 {
                self.restart_at(start_us, true)?;
            }
        } else {
            self.restart_at(
                start_us,
                self.format.is_some()
                    || self.last_observation_end.is_some_and(|end| start_us > end),
            )?;
        }
        if self.format.is_none() {
            self.format = Some(format);
            self.base_us = start_us;
            self.frame_start_us = start_us;
        }
        let end = i128::from(self.base_us)
            + (i128::from(self.input_frames) + frames as i128) * 1_000_000 / i128::from(rate);
        if end > i128::from(i64::MAX - 120_000_000) {
            return self.result(PcmStatus::InvalidTime, 0);
        }
        let samples_per_frame = (rate / 50) as usize;
        let mut emitted = 0;
        for sample in pcm.chunks_exact(2 * channels as usize) {
            let left = i16::from_le_bytes([sample[0], sample[1]]);
            let mono = if channels == 2 {
                let right = i16::from_le_bytes([sample[2], sample[3]]);
                ((i32::from(left) + i32::from(right)) / 2) as i16
            } else {
                left
            };
            self.partial[self.partial_len] = mono;
            self.partial_len += 1;
            self.input_frames += 1;
            if self.partial_len == samples_per_frame {
                let speech = self
                    .vad
                    .classify(rate, &self.partial[..samples_per_frame])
                    .map_err(|_| CoreError::Bridge)?;
                let end_us = self.frame_start_us + 20_000;
                self.observe(Observation {
                    start_us: self.frame_start_us,
                    end_us,
                    activity: if speech {
                        Activity::Speech
                    } else {
                        Activity::Silence
                    },
                })?;
                self.last_frame = Some((self.frame_start_us, end_us));
                self.frame_start_us = end_us;
                self.partial_len = 0;
                self.processed_frames += 1;
                emitted += 1;
            }
        }
        self.result(PcmStatus::Ok, emitted)
    }

    pub(super) fn gap(&mut self, start_us: i64, end_us: i64) -> Result<PcmResult, CoreError> {
        if !valid_time(start_us) || !valid_time(end_us) || end_us <= start_us {
            return self.result(PcmStatus::InvalidTime, 0);
        }
        if self.last_observation_end.is_some_and(|end| start_us < end) {
            return self.result(PcmStatus::NeedsNewEpoch, 0);
        }
        let begin = self.last_observation_end.unwrap_or(start_us);
        self.vad.reset().map_err(|_| CoreError::Bridge)?;
        self.observe(Observation {
            start_us: begin,
            end_us,
            activity: Activity::Gap,
        })?;
        self.clear_at(end_us);
        self.gaps += 1;
        self.result(PcmStatus::Ok, 0)
    }

    fn restart_at(&mut self, start_us: i64, gap: bool) -> Result<(), CoreError> {
        self.vad.reset().map_err(|_| CoreError::Bridge)?;
        if gap {
            let begin = self
                .last_observation_end
                .or_else(|| (self.partial_len > 0).then_some(self.frame_start_us));
            if let Some(begin) = begin.filter(|begin| *begin < start_us) {
                self.observe(Observation {
                    start_us: begin,
                    end_us: start_us,
                    activity: Activity::Gap,
                })?;
            }
            self.gaps += 1;
        }
        self.clear_at(start_us);
        Ok(())
    }

    fn clear_at(&mut self, start_us: i64) {
        self.partial_len = 0;
        self.format = None;
        self.input_frames = 0;
        self.base_us = start_us;
        self.frame_start_us = start_us;
    }

    fn observe(&mut self, observation: Observation) -> Result<(), CoreError> {
        self.estimator
            .push(self.epoch, &[observation])
            .map_err(|_| CoreError::InvalidInput)?;
        self.last_observation_end = Some(observation.end_us);
        Ok(())
    }

    fn result(&mut self, status: PcmStatus, emitted_frames: u32) -> Result<PcmResult, CoreError> {
        let estimate: Estimate = self
            .estimator
            .estimate(self.epoch)
            .map_err(|_| CoreError::InvalidInput)?;
        Ok(PcmResult {
            epoch: self.epoch,
            status,
            processed_frames: self.processed_frames,
            emitted_frames,
            pending_samples: self.partial_len,
            gaps: self.gaps,
            last_frame_start_us: self.last_frame.map(|f| f.0),
            last_frame_end_us: self.last_frame.map(|f| f.1),
            estimate: Some(estimate),
        })
    }
}

fn valid_time(time: i64) -> bool {
    (0..=i64::MAX - 120_000_000).contains(&time)
}
