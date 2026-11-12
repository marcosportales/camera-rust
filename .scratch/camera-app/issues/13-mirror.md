# 13: Mirror

**What to build:** A Mirror button and M shortcut flip the Viewfinder horizontally. Saved Captures are never flipped.

**Blocked by:** 03

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/08-components.html` (Icon buttons: "mirror · off", "mirror · on").

- [ ] Mirror button right of the Shutter at 40 px (`space.40`) gap, `size::MIRROR` (48) circle (`radius::FULL`), fill `color::CONTROL_HOVER` when off as in `01-ready-photo.html`, `flip-horizontal` icon (`icon.lg`).
- [ ] Mirror is on at startup; "on" look is `color::CONTROL_ON` fill, never the red accent.
- [ ] Clicking the button, or pressing M, toggles Mirror and flips the Viewfinder image horizontally. Mirror is a view-only transform.
- [ ] A Photo taken with Mirror on is identical to one taken with Mirror off (test: the saved JPEG bytes equal the unflipped source frame).
- [ ] Mirror state is not saved between runs.
- [ ] Core tests: initial on, toggle by click and by M.

**Out of this ticket:** Locking (12), tooltip (16).
