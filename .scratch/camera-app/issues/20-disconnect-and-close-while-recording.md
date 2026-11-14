# 20: Camera disconnect and window close while recording

**What to build:** Losing the camera or closing the window during a recording finalizes the Video instead of corrupting it.

**Blocked by:** 07, 19

**Status:** ready-for-agent

**Design reference:** screens from tickets 07 and 19 (no new screen).

**GPUI references:** Window close handling: [`on_window_close_quit.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/on_window_close_quit.rs).

- [x] Closing the window while recording sends EOS, waits for the file to be finalized, and only then exits the process. The Shutter is not clickable after close was requested.
- [x] If finalization after a close fails, the process exits anyway; no dialog.
- [x] If the camera disappears while recording (device removed, pipeline error from the source), the recording stops in an orderly way: the app sends EOS and tries to finalize the partial Video. If finalized, the file stays on disk and the toast "Video saved" (ticket 07) is shown only after EOS completes; if it cannot be finalized, the failure toast from ticket 07 is shown.
- [ ] After the stop, the app shows the error screen for what happened (`NoCamera` for a removed device, otherwise by the classification rules of ticket 19), with Retry.
- [ ] If the camera disappears while **not** recording, the app goes directly to the matching error screen.
- [x] A Video is never reported as saved before EOS completion (carries over from ticket 07).
- [x] Core tests with the fake backend: close while recording; disconnect while recording with successful EOS; disconnect with failed EOS; disconnect while idle.
- [x] Integration test: with the source removed mid-recording (simulated by stopping the source element), the output MP4 is playable.

**Out of this ticket:** Recovering a recording after the camera comes back.

## Implementation notes

- **Post-stop and idle error classification — blocking, high urgency:** Known camera errors and removed V4L2 devices route to the matching Retry screen; unknown pipeline errors still inherit ticket 19's existing `NoCamera` fallback (`src/gst_camera/errors.rs:13`). The ticket explicitly requires a user decision for unknown errors, which is pending. Runtime polling (`src/gst_camera.rs:68`) uses that shared classifier. Finish ticket 19's unknown-error policy before marking these two requirements complete. Verified the known-error paths with `cargo test app_core::lifecycle` and stopped-source MP4 playback with `cargo test source_removed_mid_recording`; full `cargo test` passed 98 tests. No error dialog or new screen was invented.
