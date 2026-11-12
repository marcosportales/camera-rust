# 08: Video without a microphone

**What to build:** Recording works when no microphone exists: the Video is saved without an audio track and nothing is reported as an error.

**Blocked by:** 06

**Status:** ready-for-agent

**Design reference:** none (no UI change).

- [ ] The audio source trait reports whether a usable default input exists before recording starts.
- [ ] If no usable audio input exists, recording starts and the MP4 contains only a video stream. No toast, no label, no error state.
- [ ] If the audio source fails while recording, the recording continues video-only. No toast, no label.
- [ ] Core tests with the fake audio source: available → audio recorded; unavailable → recording starts and completes normally.
- [ ] Integration test: with the audio source forced unavailable, the output MP4 has exactly one video stream and no audio stream.

**Out of this ticket:** Choosing or listing microphones.
