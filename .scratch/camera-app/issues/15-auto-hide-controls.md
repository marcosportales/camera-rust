# 15: Auto-hide controls

**What to build:** The overlaid controls fade out after a few seconds without mouse movement and return when the mouse moves. Shortcuts keep working while hidden.

**Blocked by:** 12, 13, 14

**Status:** ready-for-agent

**Design reference:** motion tokens in `html/09-design-tokens.html` (`controls_in`, `controls_out`, `idle_timeout`).

**GPUI references:** Fade: [`animation.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/animation.rs); idle timer: [`contexts.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md).

- [x] "Controls" are: Timer button, Resolution button, mode pill, Last-Capture Thumbnail (if present), Shutter, Mirror button, and the `scrim_top`/`scrim_bottom` gradients. The camera name stays visible always.
- [ ] After `motion::IDLE_TIMEOUT` (2500 ms) with no mouse movement over the window, controls fade out over `motion::CONTROLS_OUT` (300 ms, ease-in). Mouse movement fades them in over `motion::CONTROLS_IN` (150 ms, ease-out) and restarts the idle timer.
- [x] The idle timer is paused while a menu (Timer or Resolution) is open; controls stay visible.
- [x] Controls stay fully visible during a countdown, during recording and during "Saving…" (tickets 10, 11, 06, 07), regardless of mouse movement; the idle timer starts again only when those states end.
- [ ] Hidden controls do not react to hover or clicks (a click on the Viewfinder area where a hidden control sits only wakes the controls, it does not activate the control).
- [x] Space, P, V, T, M and Esc work exactly as when visible.
- [x] The recording-timer pill and toasts are not controls: they follow their own rules and are never hidden by this.
- [x] Core tests with a controllable clock for the idle logic: hide after 2500 ms, wake on movement, paused with a menu open, never hidden in countdown/recording/saving.

**Out of this ticket:** Tooltips (16).

## Implementation notes

- **Fade rendering/timing verification — non-blocking, low urgency.** Implemented and verified the 2500 ms idle deadline, 300 ms ease-in exit, 150 ms ease-out entry, and reversal continuity through `cargo test auto_hide` (`src/app_core/auto_hide.rs:94`). UI animation frames are requested in `src/camera_window/auto_hide.rs:66`. A webcam-backed visual check remains: `ls /dev/video*` returned `ls: cannot access '/dev/video*': No such file or directory`. Verify the visible fade durations and movement over buttons/menus on a machine with a webcam before checking off this end-to-end criterion.
- **Hidden hover/click verification — non-blocking, medium urgency.** Implemented the occluding wake surface and movement observer in `src/controls_ui.rs:15` and `src/controls_ui.rs:32`; reviewed GPUI 0.2.2 hit testing and click dispatch to verify shielding and preserved root shortcuts. Interactive verification remains blocked by the same absent webcam (`ls /dev/video*`, error above). With a webcam, leave the pointer stationary over Shutter until controls hide, then click once: it must only wake the controls; confirm hover cannot reach hidden controls. Test movement and all six shortcuts too. No implementation defect is currently known.
