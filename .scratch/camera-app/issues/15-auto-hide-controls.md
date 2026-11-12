# 15: Auto-hide controls

**What to build:** The overlaid controls fade out after a few seconds without mouse movement and return when the mouse moves. Shortcuts keep working while hidden.

**Blocked by:** 12, 13, 14

**Status:** ready-for-agent

**Design reference:** motion tokens in `html/09-design-tokens.html` (`controls_in`, `controls_out`, `idle_timeout`).

- [ ] "Controls" are: Timer button, Resolution button, mode pill, Last-Capture Thumbnail (if present), Shutter, Mirror button, and the `scrim_top`/`scrim_bottom` gradients. The camera name stays visible always.
- [ ] After `motion::IDLE_TIMEOUT` (2500 ms) with no mouse movement over the window, controls fade out over `motion::CONTROLS_OUT` (300 ms, ease-in). Mouse movement fades them in over `motion::CONTROLS_IN` (150 ms, ease-out) and restarts the idle timer.
- [ ] The idle timer is paused while a menu (Timer or Resolution) is open; controls stay visible.
- [ ] Controls stay fully visible during a countdown, during recording and during "Saving…" (tickets 10, 11, 06, 07), regardless of mouse movement; the idle timer starts again only when those states end.
- [ ] Hidden controls do not react to hover or clicks (a click on the Viewfinder area where a hidden control sits only wakes the controls, it does not activate the control).
- [ ] Space, P, V, T, M and Esc work exactly as when visible.
- [ ] The recording-timer pill and toasts are not controls: they follow their own rules and are never hidden by this.
- [ ] Core tests with a controllable clock for the idle logic: hide after 2500 ms, wake on movement, paused with a menu open, never hidden in countdown/recording/saving.

**Out of this ticket:** Tooltips (16).
