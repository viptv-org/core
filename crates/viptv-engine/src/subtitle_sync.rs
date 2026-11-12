use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const ALGORITHM_VERSION: &str = "activity-offset-affine-v1";
pub const MAX_CUES: usize = 16_384;
pub const MAX_BATCH: usize = 8_192;
const BIN_US: i64 = 100_000;
const WINDOW: usize = 600;
const SEARCH: i64 = 300;
const HISTORY: i64 = 18_000;
const MAX_ANCHORS: usize = 16;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CueInterval {
    pub start_us: i64,
    pub end_us: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Activity {
    Speech,
    Silence,
    Gap,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub start_us: i64,
    pub end_us: i64,
    pub activity: Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Correction {
    pub anchor_us: i64,
    pub offset_us: i64,
    pub scale: f64,
}

impl Correction {
    pub fn title_time_us(self, subtitle_us: i64) -> Option<i64> {
        if !self.scale.is_finite() || self.scale <= 0.0 {
            return None;
        }
        let delta = i128::from(subtitle_us) - i128::from(self.anchor_us);
        rounded_time(
            i128::from(self.anchor_us) + i128::from(self.offset_us),
            delta as f64 * self.scale,
        )
    }

    pub fn subtitle_time_us(self, title_us: i64, manual_delay_us: i64) -> Option<i64> {
        if !self.scale.is_finite() || self.scale <= 0.0 {
            return None;
        }
        let delta = i128::from(title_us)
            - i128::from(self.anchor_us)
            - i128::from(self.offset_us)
            - i128::from(manual_delay_us);
        rounded_time(i128::from(self.anchor_us), delta as f64 / self.scale)
    }

    pub fn mpv_delay_us(self, origin_us: i64, manual_delay_us: i64) -> Option<i64> {
        if !self.scale.is_finite() || self.scale <= 0.0 {
            return None;
        }
        let delta = i128::from(self.anchor_us) - i128::from(origin_us);
        rounded_time(
            i128::from(self.offset_us) + i128::from(manual_delay_us),
            delta as f64 * (1.0 - self.scale),
        )
    }

    pub fn player_time_us(
        self,
        subtitle_us: i64,
        origin_us: i64,
        manual_delay_us: i64,
    ) -> Option<i64> {
        let title = self.title_time_us(subtitle_us)?;
        i64::try_from(i128::from(title) - i128::from(origin_us) + i128::from(manual_delay_us)).ok()
    }
}

fn rounded_time(base: i128, delta: f64) -> Option<i64> {
    if !delta.is_finite() || delta.abs() > i64::MAX as f64 {
        return None;
    }
    i64::try_from(base + delta.round() as i128).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Observing,
    Accepted,
    Ambiguous,
    Unavailable,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub start_us: i64,
    pub end_us: i64,
    pub observed_us: i64,
    pub speech_us: i64,
    pub coverage: f64,
    pub score: f64,
    pub peak_margin: f64,
    pub anchors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub algorithm_version: String,
    pub epoch: u64,
    pub revision: u64,
    pub state: State,
    pub correction: Option<Correction>,
    pub evidence: Evidence,
    pub correction_evidence: Option<Evidence>,
    pub confidence_kind: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SyncError {
    #[error("observation epoch does not match")]
    StaleEpoch,
    #[error("subtitle synchronization input exceeds a bound or has invalid intervals")]
    InvalidInput,
}

#[derive(Clone)]
struct Anchor {
    subtitle_us: i64,
    offset_us: i64,
    evidence: Evidence,
}

pub struct Estimator {
    epoch: u64,
    cues: Vec<CueInterval>,
    bins: VecDeque<(u32, u32)>,
    first_bin: i64,
    last_end_us: Option<i64>,
    analyzed_end: Option<i64>,
    anchors: VecDeque<Anchor>,
    output: Estimate,
}

impl Estimator {
    pub fn new(epoch: u64, cues: Vec<CueInterval>) -> Result<Self, SyncError> {
        if cues.len() > MAX_CUES
            || cues.iter().any(|c| !valid_interval(c.start_us, c.end_us))
            || cues.windows(2).any(|w| w[0].start_us > w[1].start_us)
        {
            return Err(SyncError::InvalidInput);
        }
        Ok(Self {
            epoch,
            cues,
            bins: VecDeque::new(),
            first_bin: 0,
            last_end_us: None,
            analyzed_end: None,
            anchors: VecDeque::new(),
            output: Estimate {
                algorithm_version: ALGORITHM_VERSION.into(),
                epoch,
                revision: 0,
                state: State::Observing,
                correction: None,
                evidence: Evidence::default(),
                correction_evidence: None,
                confidence_kind: "uncalibratedHeuristic".into(),
            },
        })
    }

    pub fn push(&mut self, epoch: u64, observations: &[Observation]) -> Result<(), SyncError> {
        self.check_epoch(epoch)?;
        if observations.len() > MAX_BATCH {
            return Err(SyncError::InvalidInput);
        }
        let mut last = self.last_end_us;
        let mut observed_duration = 0_i128;
        for o in observations {
            if !valid_interval(o.start_us, o.end_us) || last.is_some_and(|end| o.start_us < end) {
                return Err(SyncError::InvalidInput);
            }
            if o.activity != Activity::Gap {
                observed_duration += i128::from(o.end_us) - i128::from(o.start_us);
                if observed_duration > 180_000_000 {
                    return Err(SyncError::InvalidInput);
                }
            }
            last = Some(o.end_us);
        }
        for o in observations {
            let end = (o.end_us - 1).div_euclid(BIN_US) + 1;
            let start = o.start_us.div_euclid(BIN_US);
            if self.bins.is_empty() {
                self.first_bin = start.max(end - HISTORY);
            }
            let keep_from = (end - HISTORY).max(self.first_bin);
            let drop = (keep_from - self.first_bin).min(self.bins.len() as i64) as usize;
            self.bins.drain(..drop);
            self.first_bin = keep_from;
            while self.first_bin + (self.bins.len() as i64) < end {
                self.bins.push_back((0, 0));
            }
            for bin in start.max(self.first_bin)..end {
                let index = (bin - self.first_bin) as usize;
                let overlap =
                    (o.end_us.min((bin + 1) * BIN_US) - o.start_us.max(bin * BIN_US)) as u32;
                if o.activity != Activity::Gap {
                    self.bins[index].0 += overlap;
                    if o.activity == Activity::Speech {
                        self.bins[index].1 += overlap;
                    }
                }
            }
            self.last_end_us = Some(o.end_us);
        }
        Ok(())
    }

    pub fn estimate(&mut self, epoch: u64) -> Result<Estimate, SyncError> {
        self.check_epoch(epoch)?;
        let Some(end_us) = self.last_end_us else {
            return Ok(self.output.clone());
        };
        let end_bin = end_us.div_euclid(BIN_US);
        if self
            .analyzed_end
            .is_some_and(|end| end_bin - end < WINDOW as i64)
        {
            return Ok(self.output.clone());
        }
        if end_bin - self.first_bin < WINDOW as i64 {
            return Ok(self.output.clone());
        }
        self.analyzed_end = Some(end_bin);
        let start_bin = end_bin - WINDOW as i64;
        let skip = (start_bin - self.first_bin).max(0) as usize;
        let samples: Vec<_> = self
            .bins
            .iter()
            .skip(skip)
            .take(WINDOW)
            .map(|&(observed, speech)| {
                if observed == BIN_US as u32 {
                    Some(speech >= BIN_US as u32 / 2)
                } else {
                    None
                }
            })
            .collect();
        if samples.len() < WINDOW {
            return Ok(self.output.clone());
        }
        let observed = samples.iter().flatten().count();
        let speech = samples.iter().filter(|x| **x == Some(true)).count();
        let mut evidence = Evidence {
            start_us: start_bin * BIN_US,
            end_us: end_bin * BIN_US,
            observed_us: observed as i64 * BIN_US,
            speech_us: speech as i64 * BIN_US,
            coverage: observed as f64 / WINDOW as f64,
            anchors: self.anchors.len(),
            ..Evidence::default()
        };
        if observed < WINDOW * 3 / 4
            || speech < 60
            || observed - speech < 60
            || transitions(&samples) < 8
        {
            self.output.state = State::Unavailable;
            self.output.evidence = evidence;
            return Ok(self.output.clone());
        }
        let subtitle_start = start_bin - SEARCH;
        let cue_bins = self.cue_bins(subtitle_start, WINDOW + 2 * SEARCH as usize);
        let scores: Vec<_> = (-SEARCH..=SEARCH)
            .map(|offset| score(&samples, &cue_bins, (SEARCH - offset) as usize))
            .collect();
        let (best_index, best) = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap();
        let offset_bin = best_index as i64 - SEARCH;
        let competitor = scores
            .iter()
            .enumerate()
            .filter(|(i, _)| i.abs_diff(best_index) >= 4)
            .map(|(_, s)| *s)
            .fold(-1.0_f64, f64::max);
        evidence.score = *best;
        evidence.peak_margin = best - competitor;
        let split_consistent = [0, WINDOW / 2].iter().all(|&begin| {
            let part = &samples[begin..begin + WINDOW / 2];
            let local: Vec<_> = (-SEARCH..=SEARCH)
                .map(|offset| score(part, &cue_bins, begin + (SEARCH - offset) as usize))
                .collect();
            let (index, score) = local
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap();
            *score >= 0.55 && index.abs_diff(best_index) <= 2
        });
        if *best < 0.65 || evidence.peak_margin < 0.08 || !split_consistent {
            self.output.state = State::Ambiguous;
            self.output.evidence = evidence;
            return Ok(self.output.clone());
        }
        let offset_us = offset_bin * BIN_US;
        let title_center = start_bin * BIN_US + WINDOW as i64 * BIN_US / 2;
        self.anchors.push_back(Anchor {
            subtitle_us: title_center - offset_us,
            offset_us,
            evidence: evidence.clone(),
        });
        if self.anchors.len() > MAX_ANCHORS {
            self.anchors.pop_front();
        }
        evidence.anchors = self.anchors.len();
        let candidate = if self.output.correction.is_none() {
            Some(Correction {
                anchor_us: title_center - offset_us,
                offset_us,
                scale: 1.0,
            })
        } else {
            self.drift()
        };
        if let Some(candidate) = candidate {
            let changed = self.output.correction.is_none_or(|old| {
                let old_time = old
                    .title_time_us(candidate.anchor_us)
                    .unwrap_or(candidate.anchor_us);
                (i128::from(old_time)
                    - i128::from(candidate.anchor_us)
                    - i128::from(candidate.offset_us))
                .abs()
                    >= 100_000
                    || (old.scale - candidate.scale).abs() >= 0.00005
            });
            if changed {
                self.output.correction = Some(candidate);
                self.output.revision += 1;
                let mut accepted_evidence = evidence.clone();
                if candidate.scale != 1.0 {
                    accepted_evidence.start_us = self.anchors.front().unwrap().evidence.start_us;
                    accepted_evidence.observed_us =
                        self.anchors.iter().map(|a| a.evidence.observed_us).sum();
                    accepted_evidence.speech_us =
                        self.anchors.iter().map(|a| a.evidence.speech_us).sum();
                    accepted_evidence.coverage = accepted_evidence.observed_us as f64
                        / (self.anchors.len() * WINDOW) as f64
                        / BIN_US as f64;
                    accepted_evidence.score = self
                        .anchors
                        .iter()
                        .map(|a| a.evidence.score)
                        .fold(1.0, f64::min);
                    accepted_evidence.peak_margin = self
                        .anchors
                        .iter()
                        .map(|a| a.evidence.peak_margin)
                        .fold(1.0, f64::min);
                }
                self.output.correction_evidence = Some(accepted_evidence);
            }
        }
        self.output.state = State::Accepted;
        self.output.evidence = evidence;
        Ok(self.output.clone())
    }

    fn cue_bins(&self, start: i64, length: usize) -> Vec<bool> {
        let mut bins = vec![false; length];
        for cue in &self.cues {
            let lo = (cue.start_us.div_euclid(BIN_US) - start).clamp(0, length as i64) as usize;
            let hi = (cue.end_us.div_euclid(BIN_US) - start).clamp(0, length as i64) as usize;
            bins[lo..hi].fill(true);
        }
        bins
    }

    fn drift(&self) -> Option<Correction> {
        if self.anchors.len() < 5 {
            return None;
        }
        let span = self.anchors.back()?.subtitle_us - self.anchors.front()?.subtitle_us;
        if span < 600_000_000 {
            return None;
        }
        let origin = self.anchors.back()?.subtitle_us;
        let mut slopes = Vec::new();
        for (i, a) in self.anchors.iter().enumerate() {
            for b in self.anchors.iter().skip(i + 1) {
                if b.subtitle_us - a.subtitle_us >= 120_000_000 {
                    slopes.push(
                        (b.offset_us - a.offset_us) as f64 / (b.subtitle_us - a.subtitle_us) as f64,
                    );
                }
            }
        }
        let slope = median(&mut slopes)?;
        if slope.abs() > 0.005 {
            return None;
        }
        let mut offsets: Vec<_> = self
            .anchors
            .iter()
            .map(|a| a.offset_us as f64 - slope * (a.subtitle_us - origin) as f64)
            .collect();
        let offset = median(&mut offsets)?;
        let inliers: Vec<_> = self
            .anchors
            .iter()
            .filter(|a| {
                (a.offset_us as f64 - offset - slope * (a.subtitle_us - origin) as f64).abs()
                    <= 150_000.0
            })
            .collect();
        if inliers.len() < 5
            || inliers.len() * 4 < self.anchors.len() * 3
            || inliers.last()?.subtitle_us - inliers.first()?.subtitle_us < 600_000_000
        {
            return None;
        }
        Some(Correction {
            anchor_us: origin,
            offset_us: offset.round() as i64,
            scale: 1.0 + slope,
        })
    }

    fn check_epoch(&self, epoch: u64) -> Result<(), SyncError> {
        if epoch == self.epoch {
            Ok(())
        } else {
            Err(SyncError::StaleEpoch)
        }
    }
}

fn valid_interval(start: i64, end: i64) -> bool {
    start >= 0 && end > start && end <= i64::MAX - 120_000_000
}

fn transitions(samples: &[Option<bool>]) -> usize {
    samples
        .windows(2)
        .filter(|w| matches!((w[0],w[1]), (Some(a),Some(b)) if a != b))
        .count()
}

fn score(samples: &[Option<bool>], cues: &[bool], start: usize) -> f64 {
    let (mut a, mut b, mut c, mut d) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for (sample, cue) in samples.iter().zip(&cues[start..]) {
        match (sample, cue) {
            (Some(true), true) => a += 1.0,
            (Some(true), false) => b += 1.0,
            (Some(false), true) => c += 1.0,
            (Some(false), false) => d += 1.0,
            (None, _) => (),
        }
    }
    let denominator = ((a + b) * (a + c) * (b + d) * (c + d)).sqrt();
    if denominator == 0.0 {
        -1.0
    } else {
        (a * d - b * c) / denominator
    }
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
}
