# 06: Record a Video

**What to build:** In Video Mode, the Shutter starts and stops a recording that is written to an MP4 file, with the recording indicator. The codec/container and the audio source are traits so they can be swapped.

**Blocked by:** 05

**Status:** ready-for-agent

**Design reference:** `html/04-recording.html` / `png/04-recording.png`, `html/08-components.html` (Shutter "recording · stop").

- [x] Two traits exist: one for the Video encoder + container, one for the audio source. The core and the GStreamer backend depend on the traits, not on concrete element names.
- [x] First implementations: encoder `openh264enc` (H.264) with `avenc_aac` (AAC) in `mp4mux`; audio source `pulsesrc` on the system default device. `x264enc` is not installed and must not be used.
- [x] In Video Mode, Shutter click or Space starts recording; pressing again stops it. Video is recorded at the active Resolution.
- [x] Saved to the XDG Videos directory plus `Camera` (fallback `~/Videos/Camera`), created when missing, named `YYYY-MM-DD_HH-MM-SS.mp4` with the same `_1`, `_2` collision rule as Photos.
- [x] While recording, the screen matches `04-recording.html`: only the camera name (top left), the recording timer (top center) and the Shutter are drawn. The mode pill, Timer, Resolution, Mirror and Thumbnail are not drawn.
- [x] Shutter while recording: ring unchanged; fill replaced by a stop square `size::STOP` (26), `radius::STOP`, color `RECORD` (pressed `RECORD_PRESSED`).
- [x] Recording timer: `size::STATUS` (32) tall, `radius::FULL`, `color::CONTROL` fill, `shadow_control`; an 8 px `RECORD` dot whose opacity pulses 1 ↔ 0.35 over `motion::REC_PULSE` (1000 ms); elapsed time in `type.timecode` with tabular figures, format `MM:SS` (minutes keep counting past 59, e.g. `61:05`), updated every second starting at `00:00`.
- [x] When recording stops, the file is closed correctly by sending EOS through the pipeline (the finalized-confirmation UI is ticket 07; here the Shutter simply returns to its Video-Mode look after the stop).
- [x] Core tests with the fake backend: start, stop, elapsed seconds ticking, the stop event ignored when not recording.
- [x] Integration test: the real backend records 2 seconds from `videotestsrc` + `audiotestsrc` to a temp directory and the result is a playable MP4 (demuxes with video and audio streams). Skipped if `openh264enc`, `avenc_aac` or `mp4mux` is missing.

**Out of this ticket:** "Saving…" state and write-failure notice (07), no-microphone behavior (08), Timer, locking, Thumbnail, close/disconnect handling (20).

## Implementation notes

- **Traits and "the core depends on the traits" (non-blocking, urgency: low).** `VideoFormat` and `AudioSource` live in the GStreamer side (`src/gst_recording.rs:15-58`), not in `app_core`, because they return `gst-launch` fragments and the core is GStreamer-free. The core depends only on `CameraBackend::start_video`/`stop_video` (`src/app_core/backend.rs`), so it names no element; the backend's `Recorder` takes the two traits as boxes and is wired with `H264AacMp4` + `DefaultMicrophone` in `src/camera_window.rs`. The glue elements `jpegdec`, `videoconvert`, `audioconvert`, `audioresample` and `filesink` stay fixed in `Recorder::pipeline` (`src/gst_recording.rs`). Remaining: nothing unless the stricter reading (traits inside the core) is wanted.
