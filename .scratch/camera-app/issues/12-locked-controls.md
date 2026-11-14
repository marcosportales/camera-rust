# 12: Locked controls (explicit disabled state)

**What to build:** During a countdown and during a recording, Mode, Timer, Resolution and Mirror cannot be changed and look visibly disabled, with a tooltip saying why. This ticket **overrides** screens 03 and 04, which hide these controls: here they stay drawn but disabled (decision recorded in the grilling session, Q16).

**Blocked by:** 11, 13, 14

**Status:** ready-for-agent

**Design reference:** `html/08-components.html` ("disabled" state in Icon buttons and Shutter), tooltip style from `html/01-ready-photo.html`. The `disabled` token is `color::TEXT_DISABLED` for text/icons.

- [ ] While a countdown or recording is active, the mode pill (both segments), Timer button, Resolution button and Mirror button remain drawn at their normal positions in the `disabled` look: icons/text in `color::TEXT_DISABLED`, no hover fill, no pressed state, default cursor.
- [ ] Clicking them, and the shortcuts P, V, T and M, do nothing during those states, and the menus (Timer, Resolution) cannot be opened.
- [ ] Hovering a disabled control (after the standard tooltip delay) shows a tooltip in the standard tooltip style (28 px, `radius::SM`, `color::OVERLAY`, `type.small`) with the text "Unavailable during countdown" or "Unavailable while recording" depending on the state. No keycap is shown on these tooltips.
- [ ] The Last-Capture Thumbnail is not a locked control (it only opens the folder) and keeps working.
- [ ] When the countdown ends in cancel, the Photo is taken, or the recording ends, all four controls return to normal immediately.
- [x] The recording-timer pill (ticket 06) is unaffected.
- [ ] Core tests: every Mode/Timer/Resolution/Mirror event is ignored in countdown and recording states and accepted otherwise.

**Out of this ticket:** Control fade-out (15), the standard shortcut tooltips (16).

## Implementation notes

- **Disabled appearance and explanatory tooltips — partial, blocking, high urgency.** Implemented persistent Mode/Timer controls with disabled text/icons, removed click/hover behavior while locked, and added the 500 ms GPUI tooltip with the specified messages, 28 px height, radius, overlay, and no keycap (`src/mode_pill.rs:31`, `src/timer_ui.rs:37`, `src/locked_controls.rs:12`). Mirror UI (13) and Resolution UI (14) do not exist on the baseline, so their visuals/tooltips remain blocked. User screenshots verify the recording lock and hover explanation. They also exposed washed-out disabled text on the selected Mode segment: `src/mode_pill.rs:70` now uses `color::CONTROL_ON` instead of the white selected fill while locked. Visual confirmation of this contrast fix remains pending (non-blocking, medium urgency); this environment has no `/dev/video*`. Finish prerequisites and verify their controls when available.
- **Click/shortcut/menu suppression and core event coverage — partial, blocking, high urgency.** `AppState::handle` ignores every existing settings event during countdown/recording, including Mirror, and closes the Timer menu on capture start (`src/app_core/controls.rs:35`, `src/app_core/state.rs:240`). P/V/T and clicks use this guard. `cargo test locked_controls` verifies suppression and restoration. Resolution selection/menu events and the M shortcut are absent pending 14 and 13; wire them to this lock and extend tests when those tickets land.
- **Immediate restoration — partial, blocking, medium urgency.** Existing controls return after cancellation, countdown completion, and `RecordingStopped`, including while the Video finalizes; settings may change during saving while the Shutter remains blocked (`src/camera_window.rs:212`, `src/app_core/state.rs:229`, `src/app_core/controls.rs:85`). Verified via core tests; Mirror/Resolution UI restoration and hardware verification remain pending the prerequisites above.
