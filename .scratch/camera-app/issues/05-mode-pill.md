# 05: Mode pill (Photo | Video)

**What to build:** Two separate option buttons that switch the Mode, with P and V shortcuts and the Shutter color following the Mode.

**Blocked by:** 04

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/02-ready-video.html`, `html/08-components.html` (Mode pill, Shutter "video" states).

**GPUI references:** Key bindings (P, V): [`input.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/input.rs), [`key_dispatch.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md).

- [ ] Pill above the Shutter, `space.16` gap, centered, track `size::PILL` (36) tall, `radius::FULL`, fill `color::CONTROL`, `shadow_control`. Two segments labelled "Photo" and "Video", each `size::SEGMENT` (32) tall, horizontal padding 16, inset `space.2`, as in the design.
- [ ] Active segment: fill `color::SELECTED`, text `TEXT_ON_SELECTED`, `type.label` weight 600. Inactive: transparent, `type.label` weight 500, text color as in `01-ready-photo.html`. Inactive hover: fill `ITEM_HOVER`. Fade `motion::MODE_SWITCH` (150 ms) for the segment fill.
- [ ] Clicking a segment sets the Mode. P selects Photo, V selects Video. Selecting the already active Mode does nothing.
- [ ] Shutter fill by Mode: Photo = `SHUTTER_FILL` states (ticket 04); Video = `RECORD`, hover `RECORD_HOVER`, pressed `RECORD_PRESSED`. The change fades over 150 ms. The ring stays `SHUTTER_RING` in both.
- [ ] In Video Mode, pressing the Shutter or Space does nothing in this ticket (recording is ticket 06).
- [ ] Core tests: P/V/click events set the Mode and re-selecting is a no-op.

**Out of this ticket:** Tooltips ("Video mode · V" in `02-ready-video`), recording, locking during countdown/recording.
