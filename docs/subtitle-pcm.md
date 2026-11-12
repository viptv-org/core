# Native subtitle PCM frontend

Enable core feature `subtitle-vad` to compile the pinned native WebRTC VAD.
`subtitlePcmCapabilities()` reports availability from the actual loaded library.
The exported ABI remains present without that feature and reports unavailable.
Generated bindings and the native library must be adopted from the same core
revision; build native bindings with `VIPTV_CORE_FEATURES=native,subtitle-vad`.
Require `frontendVersion=pcm16-webrtc20-v2` and
`vadPatch=48k-input-offset-v1` before activating the worker. These identify the
48 kHz correction that consumes every 10 ms input block.

Use `SubtitlePcmSyncBridge(epoch, cuesJson)` on one background worker:

```text
pushPcm16(epoch, startUs, sampleRate, channels, "pcmS16Le", pcmBytes)
gap(epoch, startUs, endUs)
```

PCM is signed little-endian 16-bit interleaved audio, mono or stereo, at
8/16/32/48 kHz. Each call accepts at most 32,768 bytes with whole interleaved
sample frames. Stereo uses an overflow-safe integer mean. There is no arbitrary
resampling, channel-layout inference, encoded-audio decode or energy classifier.
The frontend accumulates exact 20 ms mono frames; at most 960 pending samples
are retained. Timestamp arithmetic uses integer sample counts from a stream
anchor, preserving partial-frame clocks at large timestamps. One microsecond
of forward rounding tolerance is accepted for contiguous blocks.

Only a worker may call the bridge. Playback callbacks copy into a bounded queue
and return immediately; never invoke VAD/JSON/FFI work from the callback. The
bridge has no PCM queue and uses a nonblocking mutex. `busy` means no processing
occurred; the worker must later report the dropped span through `gap`.

`gap` discards pending samples, resets VAD history and records unknown activity;
loss is never converted to silence. Forward timestamp jumps and format changes
also reset continuity. Unsupported format/rate/layout, malformed and oversized
buffers discard partial continuity and return explicit status. The caller keeps
passthrough/offload untouched and reports unavailable where PCM cannot be tapped.
Backward or overlapping already-classified time returns `needsNewEpoch`; create
a new bridge and epoch after seeks/discontinuities, source/audio/subtitle changes.
Do not send old-epoch PCM to a new bridge.

Each response contains `epoch`, `status`, cumulative `processedFrames`, per-call
`emittedFrames`, `pendingSamples`, continuity-reset counter `gaps`, the last
classified frame timestamps, and an optional activity-estimator `estimate`.
`ok` describes PCM processing; accepted correction state remains inside the
estimate. Non-ok estimates are diagnostic; do not adopt corrections from them.
Manual freeze, exact track identity, observation/correction revision
fencing and atomic correction application remain caller policies described in
[subtitle-sync.md](subtitle-sync.md). Unsupported/default-disabled responses are
not permission to change audio output mode or upload native audio to a server.

Synthetic native tests exercise WebRTC rate/frame/reset behavior, integer clocks,
stereo cancellation, partial frames, queue-loss gaps and nonblocking ownership.
They establish mechanics only. Native playback access, codec/output variants,
minimum/current OS, held-out speech/subtitle accuracy, sustained CPU/memory and
battery qualification remain required before broad automatic application.
