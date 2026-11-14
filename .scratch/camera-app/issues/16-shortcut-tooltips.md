# 16: Shortcut tooltips

**What to build:** Hovering a control shows a tooltip with its name and keycap, replacing any help screen.

**Blocked by:** 15

**Status:** ready-for-agent

**Design reference:** tooltip in `html/01-ready-photo.html` (Timer · T), `html/02-ready-video.html` (Video mode · V), `html/07a-timer-menu.html` ("Cycle timer · T"), `html/08-components.html` (Tooltip & keycaps).

- [x] Tooltip style: `size::TOOLTIP` (28) tall, `radius::SM`, `color::OVERLAY`, 1 px `border_subtle`, `shadow_overlay`, `type.small`; padding 0 4 0 12; 8 px gap to the keycap; keycap `size::KEYCAP` (20) tall, min width 20, `radius::XS`, `color::KEYCAP` fill, 1 px `KEYCAP_BORDER`, `type.caption`.
- [ ] Appears 8 px (`space.8`) below top-row controls and above bottom-row controls, centered on the control, after `motion::TOOLTIP_DELAY` (500 ms) of hover then a 100 ms fade-in; disappears immediately when the pointer leaves or the control is pressed.
- [ ] Exact texts and keycaps: Shutter in Photo Mode "Take photo" + `Space`; Shutter in Video Mode "Record video" + `Space`; Shutter while recording "Stop recording" + `Space`; Shutter during countdown "Cancel" + `Esc`; Photo segment "Photo mode" + `P`; Video segment "Video mode" + `V`; Timer button "Timer" + `T`; Timer menu row for the cycle hint "Cycle timer" + `T`; Mirror "Mirror" + `M`; Resolution button "Resolution" (no keycap); Last-Capture Thumbnail "Open folder" (no keycap).
- [x] Locked controls keep the tooltip of ticket 12 instead of these.
- [x] No tooltip is shown while controls are hidden or while a menu is open over that control.

**Out of this ticket:** Any new shortcut, any help screen.


## Implementation notes

- **Placement, delay, fade and dismissal — verification incomplete; blocking ticket sign-off; urgency medium.** Implemented an 8 px centered anchor, cancellable 500 ms hover task, 100 ms opacity fade, and mouse/keyboard dismissal in `src/tooltip_ui.rs:47`, `src/tooltip_ui.rs:80`, and `src/tooltip_ui.rs:120`. Static specification review found and resolved settings-hitbox occlusion and keyboard-dismissal issues. Live appearance/timing could not be verified: `ls /dev/video*` returned `ls: cannot access '/dev/video*': No such file or directory`; `xdotool getdisplaygeometry` returned `Failed creating new xdo instance.` Remaining: hover and press every control on a camera-equipped desktop; check top/bottom placement, delay, fade, leave/press cancellation, and hidden/menu coverage visually. Core behavior passes `cargo test shortcut_tooltips` (4 tests); `cargo test` passes all 86 tests, `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
- **Exact text/keycaps — Thumbnail integration pending; blocking this checklist item; urgency medium.** All existing controls are wired, and the exact copy is verified through AppState in `src/app_core/tooltips/tests.rs:8`. The `Open folder` description exists at `src/app_core/tooltips.rs:72`, but there is no Last-Capture Thumbnail view yet: ticket `.scratch/camera-app/issues/17-last-capture-thumbnail-photo.md:1` remains unimplemented. Remaining: when ticket 17 adds its button, wrap it with `tooltip_ui::control`, obtain `Control::Thumbnail` through `CameraWindow::shortcut_hint` (so hidden controls suppress it), and use `Placement::Above`; verify hover and folder activation together. This dependency does not block the tooltips on existing controls.
