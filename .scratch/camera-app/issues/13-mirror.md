# 13: Mirror

**What to build:** A Mirror button and M shortcut flip the Viewfinder horizontally. Saved Captures are never flipped.

**Blocked by:** 03

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/08-components.html` (Icon buttons: "mirror · off", "mirror · on").

**GPUI references:** Key binding (M): [`key_dispatch.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md).

- [ ] Mirror button right of the Shutter at 40 px (`space.40`) gap, `size::MIRROR` (48) circle (`radius::FULL`), fill `color::CONTROL_HOVER` when off as in `01-ready-photo.html`, `flip-horizontal` icon (`icon.lg`).
- [x] Mirror is on at startup; "on" look is `color::CONTROL_ON` fill, never the red accent.
- [ ] Clicking the button, or pressing M, toggles Mirror and flips the Viewfinder image horizontally. Mirror is a view-only transform.
- [x] A Photo taken with Mirror on is identical to one taken with Mirror off (test: the saved JPEG bytes equal the unflipped source frame).
- [x] Mirror state is not saved between runs.
- [x] Core tests: initial on, toggle by click and by M.

**Out of this ticket:** Locking (12), tooltip (16).

## Implementation notes

- **Interactive webcam verification — pending, non-blocking, low urgency.** `ls /dev/video*` returns `No such file or directory`, so a live webcam session cannot be exercised here. The button geometry/colors and shared click/M wiring were verified by source review (`src/mirror_ui.rs:12`, `src/camera_window.rs:113`, `src/main.rs:38`); `cargo test mirror` verifies the shortcut binding, core toggles, restart default, displayed pixel order, immediate frame reversal, and saved JPEG byte equality. `cargo test` passes all 71 tests. What remains is a manual check of the button placement and click/M interaction with a real camera on Wayland.
