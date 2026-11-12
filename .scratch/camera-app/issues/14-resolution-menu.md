# 14: Resolution menu

**What to build:** A Resolution button and menu list what the camera supports and switch the Viewfinder to the chosen size.

**Blocked by:** 03

**Status:** ready-for-agent

**Design reference:** `html/07b-resolution-menu.html` / `png/07b-resolution-menu.png`, `html/08-components.html` ("label button", "menu open").

**GPUI references:** Menu: [`popover.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/popover.rs).

- [ ] Resolution button right of the Timer button (`space.8` gap), `size::CONTROL` (40) tall, `radius::MD`, label = `<height>p` in `type.small` weight 600 (e.g. `720p`) and a `chevron-down` icon (`icon.sm`), as in `01-ready-photo.html`.
- [ ] Clicking opens a menu below it as in `07b`: width 232, `radius::LG`, `color::OVERLAY`, `shadow_overlay`, 4 px inset, 32 px rows, a header "Resolution" in `type.caption`. Each row shows the label `<height>p` and the size `W × H`; Resolutions that are not 16:9 (rule in ticket 02) append ` · 4:3` only when `W:H` is 4:3, otherwise ` · W:H` reduced to lowest terms. A `check` icon marks the current one. Rows are sorted highest pixel count first. 120 ms fade.
- [ ] Rows come only from the camera's reported MJPG Resolutions; no fixed list. The design's 1080p entry appears only if the camera reports it.
- [ ] Choosing a row selects it, closes the menu and restarts the Viewfinder at that Resolution. Clicking outside or Esc closes without change.
- [ ] If the restart fails, the Viewfinder returns to the previous Resolution and the selection reverts.
- [ ] Core tests: list order and labels for the reference camera (1280×720, 960×540, 848×480, 640×480, 640×360); selection; revert on failure.

**Out of this ticket:** Locking (12), tooltip (16), persistence.
