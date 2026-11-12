# 11: Timer for Video

**What to build:** The Timer also delays the start of a Video recording.

**Blocked by:** 06, 10

**Status:** ready-for-agent

**Design reference:** `html/03-timer-countdown.html` (same screen as Photo).

- [ ] In Video Mode with Timer 3s or 10s, pressing the Shutter or Space shows the same countdown as ticket 10, and recording starts when it ends.
- [ ] Esc or pressing the Shutter/Space again during the countdown cancels it; no file is created.
- [ ] Recording start time for the elapsed timer is the end of the countdown, not the press.
- [ ] Stopping a recording is never delayed by the Timer.
- [ ] Core tests with a controllable clock: countdown then StartRecording; cancel creates no recording; stop is immediate.

**Out of this ticket:** Locking (12).
