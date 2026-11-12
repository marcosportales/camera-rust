# 16: Shortcut tooltips

**What to build:** Hovering a control shows a tooltip with its name and keycap, replacing any help screen.

**Blocked by:** 15

**Status:** ready-for-agent

**Design reference:** tooltip in `html/01-ready-photo.html` (Timer · T), `html/02-ready-video.html` (Video mode · V), `html/07a-timer-menu.html` ("Cycle timer · T"), `html/08-components.html` (Tooltip & keycaps).

- [ ] Tooltip style: `size::TOOLTIP` (28) tall, `radius::SM`, `color::OVERLAY`, 1 px `border_subtle`, `shadow_overlay`, `type.small`; padding 0 4 0 12; 8 px gap to the keycap; keycap `size::KEYCAP` (20) tall, min width 20, `radius::XS`, `color::KEYCAP` fill, 1 px `KEYCAP_BORDER`, `type.caption`.
- [ ] Appears 8 px (`space.8`) below top-row controls and above bottom-row controls, centered on the control, after `motion::TOOLTIP_DELAY` (500 ms) of hover then a 100 ms fade-in; disappears immediately when the pointer leaves or the control is pressed.
- [ ] Exact texts and keycaps: Shutter in Photo Mode "Take photo" + `Space`; Shutter in Video Mode "Record video" + `Space`; Shutter while recording "Stop recording" + `Space`; Shutter during countdown "Cancel" + `Esc`; Photo segment "Photo mode" + `P`; Video segment "Video mode" + `V`; Timer button "Timer" + `T`; Timer menu row for the cycle hint "Cycle timer" + `T`; Mirror "Mirror" + `M`; Resolution button "Resolution" (no keycap); Last-Capture Thumbnail "Open folder" (no keycap).
- [ ] Locked controls keep the tooltip of ticket 12 instead of these.
- [ ] No tooltip is shown while controls are hidden or while a menu is open over that control.

**Out of this ticket:** Any new shortcut, any help screen.
