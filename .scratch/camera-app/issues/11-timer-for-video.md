# 11: Timer for Video

**What to build:** The Timer also delays the start of a Video recording.

**Blocked by:** 06, 10

**Status:** ready-for-agent

**Design reference:** `html/03-timer-countdown.html` (same screen as Photo).

- [x] In Video Mode with Timer 3s or 10s, pressing the Shutter or Space shows the same countdown as ticket 10, and recording starts when it ends.
- [x] Esc or pressing the Shutter/Space again during the countdown cancels it; no file is created.
- [x] Recording start time for the elapsed timer is the end of the countdown, not the press.
- [x] Stopping a recording is never delayed by the Timer.
- [x] Core tests with a controllable clock: countdown then StartRecording; cancel creates no recording; stop is immediate.

**Out of this ticket:** Locking (12).

## Implementation notes

- Non-blocking, low urgency: interactive webcam validation was not performed in this session. Core behavior is verified by `cargo test` (64 passed); Shutter/Space and Esc reuse the existing UI routes in `src/camera_window.rs:174` and `src/camera_window.rs:184`. Countdown completion dispatch is in `src/camera_window/countdown.rs:46`. Remaining: manually try Video with 3s/10s, each cancellation input, elapsed time starting at 00:00, and immediate stop.
