# Subtitle synchronization estimator

`viptv_engine::subtitle_sync::Estimator` accepts canonical subtitle cue intervals
and monotonically timestamped `Speech`, observed `Silence`, or `Gap` intervals
in title-time microseconds. A missing interval is a gap. Contiguous small frames
accumulate into 100 ms bins; partially observed bins are excluded. Supply the
selected audio track's title timestamps, including its authoritative timeline
projection. The estimator neither decodes audio nor determines speech activity.

Create a fresh estimator/`SubtitleSyncBridge` for each source, audio, subtitle or
timeline epoch. Exact media/audio/subtitle identity and manual freeze belong to
the caller. Stale epochs and invalid batches fail before mutation. A seek that
moves observations backward needs a fresh instance; correction reuse is a
caller policy. Nothing persists audio, subtitles or corrections.

## Bounds and acceptance

- At most 16,384 sorted cue intervals and 8,192 observations per batch; at most
  180 seconds of non-gap observations in a batch.
- At most 30 minutes of activity bins and 16 independent anchors. Cue storage is
  bounded separately. Sparse input jumps do not allocate intervening history.
- One 60-second window per 60 seconds of new title time, a fixed ±30-second
  offset search in 100 ms steps, and two independent half-window checks.
- Require 75% observed coverage, at least six seconds each of speech and
  silence, eight observed transitions, correlation ≥0.65, and a ≥0.08 margin
  against peaks at least 400 ms away. Both halves need 75% coverage, three
  seconds each of speech and silence,
  four transitions, correlation ≥0.55, and agreement within 200 ms.
  Search-edge peaks abstain. These are provisional synthetic thresholds.
- Drift requires at least five anchors with at least ten minutes of leverage,
  a median pairwise slope within ±0.5%, and at least 75% inliers within 150 ms.
  Inliers must also span ten minutes. A residual/quantization sensitivity bound
  must be ≤500 ppm; it is a heuristic bound, not a statistical interval.
  Revisions use 100 ms/50 ppm hysteresis.
- A validated local-offset jump exceeding 500 ms plus 0.5% of elapsed content
  time latches ambiguity for that estimator instance. Later windows cannot
  outvote a detected nonlinear edit into a global affine correction.

`state` reports the latest observation attempt. `observationRevision` increases
only when a new independent window is analyzed; use it to discard asynchronous
state responses older than the last accepted attempt in the same epoch.
`revision` increases only when the accepted correction changes. `correction`,
`revision` and `correctionEvidence` retain the last accepted revision when new
evidence is
ambiguous/unavailable. `evidence` describes only the latest attempted window.
A strong local offset that disagrees with the retained correction by more than
250 ms reports `observing` while drift evidence accumulates. Repeated estimates
without a new independent window return the same result.
`confidenceKind=uncalibratedHeuristic`: `score` is the Matthews correlation
coefficient and `peakMargin` is a correlation difference, neither a calibrated
probability. Gap/coverage failures cannot add anchors or revise a correction.
A high activity correlation cannot prove the selected audio language or release
matches: caller eligibility checks and held-out evaluation are required.

## Applying a correction

Positive `offsetUs` delays subtitles. With canonical subtitle time `s`, one
source-to-player origin `O` and separate additive manual delay `m`:

```text
t = anchor + scale * (s - anchor) + offset
p = t - O + m
s_lookup = anchor + (p + O - anchor - offset - m) / scale
```

`Correction` exposes checked title/player/inverse mapping and the corresponding
mpv delay calculation. It subtracts integer timestamps before converting local
differences to floating point, preserving small offsets at large timestamps.
Playback speed does not multiply content-time corrections. Native property
order/sign, browser cue updates and audio acquisition need platform execution.
Never apply a correction over a manual override or restart video to apply it.

## Hosts

Rust hosts use the engine directly. The UniFFI facade is
`SubtitleSyncBridge(epoch, cuesJson).push(epoch, observationsJson)`; use a fresh
object on epoch changes. JSON fields use camelCase and enum values use camelCase
(`speech`, `silence`, `gap`, `observing`, `accepted`, `ambiguous`, `unavailable`).
Bridge JSON strings are capped at 1 MiB.

The offline executable example accepts at most 4 MiB on stdin:

```sh
cargo run --locked -p viptv-engine --example subtitle_sync < input.json
```

```json
{"epoch":1,"cues":[{"startUs":1000000,"endUs":2000000}],"batches":[[{"startUs":0,"endUs":1000000,"activity":"silence"}]]}
```

It accepts at most 32 batches and emits the final estimate as one JSON object.
`subtitle_sync_bench` measures push/estimate windows using generated activity;
latency excludes fixture construction, audio acquisition, decode/VAD, bindings
and playback. `processPeakRssKiB` reads Linux process high-water RSS, including
fixture construction and all benchmark data; it is not incremental playback
memory. Other platforms report null for that metric. The implementation is independently written and adds no dependencies
or third-party alignment code.

## Qualification

Deterministic synthetic fixtures cover sign, ambiguity, mismatched activity,
different edits, gaps, epoch fencing, large timestamps, manual/origin mapping,
and affine drift over ten minutes. They establish algorithm mechanics only.
No held-out speech/subtitle accuracy, false-correction rate, coverage, platform
resource, native acquisition, playback continuity or battery target is qualified.
Production automatic application requires those gates from the workspace
subtitle-sync research, including wrong-language and music negatives.
