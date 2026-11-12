use serde_json::Value;
use viptv_core::{SubtitlePcmSyncBridge, subtitle_pcm::subtitle_pcm_capabilities};

fn call(
    b: &SubtitlePcmSyncBridge,
    epoch: u64,
    start: i64,
    rate: u32,
    channels: u8,
    bytes: Vec<u8>,
) -> Value {
    serde_json::from_str(
        &b.push_pcm16(epoch, start, rate, channels, "pcmS16Le".into(), bytes)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn capabilities_and_epoch_are_explicit() {
    let c: Value = serde_json::from_str(&subtitle_pcm_capabilities()).unwrap();
    assert_eq!(c["available"], cfg!(feature = "subtitle-vad"));
    assert_eq!(c["maxInputBytes"], 32768);
    let b = SubtitlePcmSyncBridge::new(3, "[]".into()).unwrap();
    let stale = call(&b, 2, 0, 48000, 2, vec![0; 3840]);
    assert_eq!(stale["status"], "staleEpoch");
    assert_eq!(stale["epoch"], 3);
    #[cfg(not(feature = "subtitle-vad"))]
    assert_eq!(
        call(&b, 3, 0, 48000, 2, vec![0; 3840])["status"],
        "unavailable"
    );
}

#[cfg(feature = "subtitle-vad")]
#[test]
fn partial_frames_preserve_large_integer_timestamps() {
    let b = SubtitlePcmSyncBridge::new(4, "[]".into()).unwrap();
    let base = 8_000_000_000_000_000_003;
    let a = call(&b, 4, base, 48000, 1, vec![0; 512 * 2]);
    assert_eq!(a["pendingSamples"], 512);
    assert_eq!(a["processedFrames"], 0);
    let a = call(&b, 4, base + 10_666, 48000, 1, vec![0; 448 * 2]);
    assert_eq!(a["processedFrames"], 1);
    assert_eq!(a["lastFrameStartUs"], base);
    assert_eq!(a["lastFrameEndUs"], base + 20_000);
    assert_eq!(
        call(&b, 4, base + 20_000, 48000, 1, vec![0; 2])["pendingSamples"],
        1
    );
    assert_eq!(
        call(&b, 4, base + 20_020, 48000, 1, vec![0; 959 * 2])["lastFrameEndUs"],
        base + 40_000
    );
}

#[cfg(feature = "subtitle-vad")]
#[test]
fn stereo_antiphase_is_observed_silence_at_48k() {
    let b = SubtitlePcmSyncBridge::new(4, "[]".into()).unwrap();
    let bytes: Vec<_> = (0..960)
        .flat_map(|i| {
            let left = if i % 2 == 0 { 32767_i16 } else { -32767_i16 };
            [left.to_le_bytes(), (-left).to_le_bytes()].concat()
        })
        .collect();
    let mut a = Value::Null;
    for i in 0..3000 {
        a = call(&b, 4, i * 20_000, 48000, 2, bytes.clone());
    }
    assert_eq!(a["processedFrames"], 3000);
    assert_eq!(a["gaps"], 0);
    assert_eq!(a["estimate"]["evidence"]["observedUs"], 60_000_000);
    assert_eq!(a["estimate"]["evidence"]["speechUs"], 0);
    assert!(a["estimate"]["correction"].is_null());
}

#[cfg(feature = "subtitle-vad")]
#[test]
fn gaps_and_format_changes_drop_partial_audio() {
    let b = SubtitlePcmSyncBridge::new(1, "[]".into()).unwrap();
    assert_eq!(
        call(&b, 1, 0, 48000, 2, vec![0; 480 * 4])["pendingSamples"],
        480
    );
    let gap: Value = serde_json::from_str(&b.gap(1, 10_000, 100_000).unwrap()).unwrap();
    assert_eq!(gap["pendingSamples"], 0);
    assert_eq!(gap["gaps"], 1);
    assert_eq!(
        call(&b, 1, 100_000, 16000, 1, vec![0; 320 * 2])["lastFrameStartUs"],
        100_000
    );
    let next = call(&b, 1, 200_000, 16000, 1, vec![0; 320 * 2]);
    assert_eq!(next["gaps"], 2);
    assert_eq!(next["processedFrames"], 2);
    let backwards = call(&b, 1, 0, 16000, 1, vec![0; 640]);
    assert_eq!(backwards["status"], "needsNewEpoch");
    assert_eq!(backwards["processedFrames"], 2);
}

#[cfg(feature = "subtitle-vad")]
#[test]
fn unsupported_modes_and_bad_buffers_do_not_process_pcm() {
    let b = SubtitlePcmSyncBridge::new(1, "[]".into()).unwrap();
    for (rate, channels, bytes, status) in [
        (44100, 2, vec![0; 100], "unsupportedRate"),
        (48000, 6, vec![0; 96], "unsupportedChannels"),
        (48000, 2, vec![0; 3], "invalidBuffer"),
        (48000, 2, vec![0; 32772], "inputTooLarge"),
    ] {
        let a = call(&b, 1, 0, rate, channels, bytes);
        assert_eq!(a["status"], status);
        assert_eq!(a["processedFrames"], 0);
        assert_eq!(a["pendingSamples"], 0);
    }
    let a: Value = serde_json::from_str(
        &b.push_pcm16(1, 0, 48000, 2, "aac".into(), vec![0; 64])
            .unwrap(),
    )
    .unwrap();
    assert_eq!(a["status"], "unsupportedFormat");
}

#[cfg(feature = "subtitle-vad")]
#[test]
fn dropped_subframes_cannot_make_fake_complete_frames() {
    let b = SubtitlePcmSyncBridge::new(1, "[]".into()).unwrap();
    assert_eq!(call(&b, 1, 0, 8000, 1, vec![0; 160])["pendingSamples"], 80);
    let a = call(&b, 1, 20_000, 8000, 1, vec![0; 160]);
    assert_eq!(a["processedFrames"], 0);
    assert_eq!(a["pendingSamples"], 80);
    assert_eq!(a["gaps"], 1);
    let a = call(&b, 1, 30_000, 8000, 1, vec![0; 160]);
    assert_eq!(a["processedFrames"], 1);
    assert_eq!(a["lastFrameStartUs"], 20_000);
    assert_eq!(a["lastFrameEndUs"], 40_000);
}
