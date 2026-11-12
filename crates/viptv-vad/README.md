# Native WebRTC VAD

Native C activity classification from `wiseman/py-webrtcvad` revision
`e283ca41df3a84b0e87fb1f5cb9b21580a286b09`. Only the native C VAD/SPL closure is
compiled; the Python wrapper, C++ implementation and external subprocesses are
not used. `vendor/source-manifest.json` records the source archive hash and every
upstream and compiled file hashes. Native notices are retained. One local patch,
`48k-input-offset-v1`, advances the 48 kHz input for each 10 ms resampling block.
The manifest records its upstream/patched hashes and patch-file hash; the other
44 upstream files are unchanged. The patch is in
`vendor/patches/48k-input-offset.patch`.
`vendor/LICENSE` and `THIRD_PARTY_LICENSES/webrtc-vad.txt` retain the complete
upstream MIT-wrapper/BSD-native notice. No Python dependency is introduced.

The local C allocator checks allocation before initializing the pinned VAD state;
the pinned public creator dereferences its allocation before checking failure.
The initialized state is freed through the matching WebRTC API. Release-style
`NDEBUG` C compilation uses generic C processing and has no C++ runtime dependency.
Rust validates supported rates and exact 20 ms frame lengths before FFI calls.
Each classifier is owned by one serial worker; global SPL initialization uses
the pinned platform once implementation.

Synthetic silence/tone tests execute the actual native classifier and reset.
They establish rate/frame/ownership mechanics, not speech detection accuracy,
platform audio acquisition, battery cost or compatibility of other releases.
Windows/macOS/iOS and Android runtime qualification remain separate gates.
