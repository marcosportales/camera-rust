# 12: Locked controls (explicit disabled state)

**What to build:** During a countdown and during a recording, Mode, Timer, Resolution and Mirror cannot be changed and look visibly disabled, with a tooltip saying why. This ticket **overrides** screens 03 and 04, which hide these controls: here they stay drawn but disabled (decision recorded in the grilling session, Q16).

**Blocked by:** 11, 13, 14

**Status:** ready-for-agent

**Design reference:** `html/08-components.html` ("disabled" state in Icon buttons and Shutter), tooltip style from `html/01-ready-photo.html`. The `disabled` token is `color::TEXT_DISABLED` for text/icons.

- [x] While a countdown or recording is active, the mode pill (both segments), Timer button, Resolution button and Mirror button remain drawn at their normal positions in the `disabled` look: icons/text in `color::TEXT_DISABLED`, no hover fill, no pressed state, default cursor.
- [x] Clicking them, and the shortcuts P, V, T and M, do nothing during those states, and the menus (Timer, Resolution) cannot be opened.
- [x] Hovering a disabled control (after the standard tooltip delay) shows a tooltip in the standard tooltip style (28 px, `radius::SM`, `color::OVERLAY`, `type.small`) with the text "Unavailable during countdown" or "Unavailable while recording" depending on the state. No keycap is shown on these tooltips.
- [ ] The Last-Capture Thumbnail is not a locked control (it only opens the folder) and keeps working.
- [x] When the countdown ends in cancel, the Photo is taken, or the recording ends, all four controls return to normal immediately.
- [x] The recording-timer pill (ticket 06) is unaffected.
- [ ] Core tests: every Mode/Timer/Resolution/Mirror event is ignored in countdown and recording states and accepted otherwise.

**Out of this ticket:** Control fade-out (15), the standard shortcut tooltips (16).

## Implementation notes

- **Last-Capture Thumbnail — unverified, non-blocking, low urgency.** No Thumbnail exists on this branch; its implementation belongs to ticket 17. This ticket adds no lock to it. Folder-opening behavior during countdown/recording can be verified once it exists; implementing the Thumbnail is outside this ticket's scope.
- **Core tests — partial, non-blocking for the existing controls, medium urgency.** `cargo test locked_controls` passes all three tests, covering every existing Mode/Timer/Mirror settings event during countdown/recording and acceptance after cancellation, Photo capture, and Video stop (`src/app_core/controls.rs:55`, `src/app_core/controls.rs:89`). Resolution selection/menu events do not exist yet (ticket 14); extend the guard and tests when those events are introduced. The last criterion remains unchecked specifically because Resolution coverage is pending, not because any current test fails.
