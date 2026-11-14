# 10: Countdown and cancel (Photo)

**What to build:** With a Timer set, pressing the Shutter in Photo Mode runs a visible countdown, then takes the Photo; the user can cancel it.

**Blocked by:** 04, 09

**Status:** ready-for-human

**Design reference:** `html/03-timer-countdown.html` / `png/03-timer-countdown.png`.

**GPUI references:** Countdown timer and cancel (Esc): [`contexts.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md), [`key_dispatch.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md).

- [x] With Timer Off, behavior is unchanged (Photo is immediate).
- [x] With Timer 3s or 10s, pressing the Shutter or Space in Photo Mode starts a countdown from the Timer value down to 1, one tick per second. The Photo is taken when the countdown ends (no tick shown for 0).
- [ ] Screen as in `03-timer-countdown.html`: the numeral in `type.countdown` (144 / weight 300, tabular, line-height 1) at the position in the design; each new numeral fades in over `motion::TICK_FADE` (150 ms); a full-window `color::DIM` overlay over the Viewfinder; `scrim_bottom` kept; the hint "Esc" keycap + "to cancel" below the numeral as in the design.
- [ ] During the countdown the Shutter shows the cancel look: fill `SHUTTER_FILL`, an `x` icon (`icon.lg`, `stroke.bold`, `TEXT_ON_SELECTED`), ring unchanged.
- [x] Esc, or pressing the Shutter/Space again, cancels: no Photo is taken, the UI returns to the ready look, no toast.
- [ ] Only the camera name, numeral, hint and Shutter are drawn during the countdown, as in the design. (Locked controls are changed by ticket 12.)
- [x] The countdown does not restart or repeat after completion.
- [x] Core tests with a controllable clock: 3s timer yields ticks 3, 2, 1 then TakePhoto; cancel at each tick takes no Photo; Timer Off takes it immediately.

**Out of this ticket:** Countdown for Video (11), locking (12).


## Implementation notes

- Visual verification remains incomplete (non-blocking for code review; medium urgency before merge). Implemented the numeral/fade, dim layer, hint, cancel Shutter and countdown-only controls in `src/countdown_ui.rs:15`, `src/camera_window.rs:204` and `src/camera_window/shutter.rs:143`. Static review checked these against `docs/design/Camera-UI/html/03-timer-countdown.html`. `cargo run` opened Camera, but the screenshot command captured the editor instead of Camera. Attempts to focus Camera failed with `')' expected near 'title'` and `attempt to call a nil value (field 'focus')` from Hyprland. The three visual checkboxes remain open: verify placement, 150ms fades, cancel appearance and hidden controls interactively before merge. No screenshot is committed because it does not show the app.
