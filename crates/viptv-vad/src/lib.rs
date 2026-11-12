use std::{ffi::c_void, ptr::NonNull};

pub const SOURCE_REVISION: &str = "e283ca41df3a84b0e87fb1f5cb9b21580a286b09";
pub const SAMPLE_RATES: [u32; 4] = [8_000, 16_000, 32_000, 48_000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadError {
    UnsupportedRate,
    InvalidFrame,
    Unavailable,
}

unsafe extern "C" {
    fn viptv_webrtc_vad_create() -> *mut c_void;
    fn WebRtcVad_Free(instance: *mut c_void);
    fn WebRtcVad_Init(instance: *mut c_void) -> i32;
    fn WebRtcVad_set_mode(instance: *mut c_void, mode: i32) -> i32;
    fn WebRtcVad_Process(
        instance: *mut c_void,
        rate: i32,
        audio: *const i16,
        samples: usize,
    ) -> i32;
}

pub struct Vad {
    instance: NonNull<c_void>,
}

// Each instance is exclusively owned and accessed serially; WebRTC's global SPL initialization uses once.
unsafe impl Send for Vad {}

impl Vad {
    pub fn new() -> Result<Self, VadError> {
        let instance =
            NonNull::new(unsafe { viptv_webrtc_vad_create() }).ok_or(VadError::Unavailable)?;
        Ok(Self { instance })
    }

    pub fn reset(&mut self) -> Result<(), VadError> {
        if unsafe { WebRtcVad_Init(self.instance.as_ptr()) } != 0
            || unsafe { WebRtcVad_set_mode(self.instance.as_ptr(), 2) } != 0
        {
            return Err(VadError::Unavailable);
        }
        Ok(())
    }

    pub fn classify(&mut self, rate: u32, mono: &[i16]) -> Result<bool, VadError> {
        if !SAMPLE_RATES.contains(&rate) {
            return Err(VadError::UnsupportedRate);
        }
        if mono.len() != (rate / 50) as usize {
            return Err(VadError::InvalidFrame);
        }
        match unsafe {
            WebRtcVad_Process(
                self.instance.as_ptr(),
                rate as i32,
                mono.as_ptr(),
                mono.len(),
            )
        } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(VadError::Unavailable),
        }
    }
}

impl Drop for Vad {
    fn drop(&mut self) {
        unsafe { WebRtcVad_Free(self.instance.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_vad_supports_exact_twenty_ms_at_all_rates() {
        let mut vad = Vad::new().unwrap();
        for rate in SAMPLE_RATES {
            vad.reset().unwrap();
            assert_eq!(
                vad.classify(rate, &vec![0; (rate / 50) as usize]),
                Ok(false)
            );
            assert_eq!(
                vad.classify(rate, &vec![0; (rate / 50) as usize - 1]),
                Err(VadError::InvalidFrame)
            );
        }
        assert_eq!(
            vad.classify(44_100, &vec![0; 882]),
            Err(VadError::UnsupportedRate)
        );
    }

    #[test]
    fn reset_clears_classifier_history_after_synthetic_signal() {
        let mut vad = Vad::new().unwrap();
        let tone: Vec<_> = (0..320)
            .map(|i| {
                ((i as f64 * std::f64::consts::TAU * 440.0 / 16_000.0).sin() * 20_000.0) as i16
            })
            .collect();
        let decisions: Vec<_> = (0..10)
            .map(|_| vad.classify(16_000, &tone).unwrap())
            .collect();
        assert!(decisions.iter().any(|speech| *speech));
        vad.reset().unwrap();
        assert_eq!(vad.classify(16_000, &[0; 320]), Ok(false));
    }
}
