# 09: Timer setting (button, menu, T shortcut)

**What to build:** The Timer button opens a menu to choose Off / 3 seconds / 10 seconds, and T cycles the setting. The countdown itself comes later; this ticket only sets and shows the Timer.

**Blocked by:** 05

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/07a-timer-menu.html` / `png/07a-timer-menu.png`, `html/08-components.html` (Icon buttons, Menu).

**GPUI references:** Key binding (T) and menu: [`key_dispatch.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md), [`popover.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/popover.rs).

- [x] Timer button, top right (inset `space.16`), `size::CONTROL` (40) square, `radius::MD`, `color::CONTROL_HOVER` fill as in `01-ready-photo.html`, `shadow_control`, `timer` icon (`icon.lg`). Layout order with future top-right controls follows the design: Timer button, then Resolution button, 8 px apart (`space.8`).
- [x] When the Timer is 3s or 10s, the button shows its "set" look from `07a-timer-menu.html` (`color::CONTROL_ON` fill with the value text "3s"/"10s"). When Off it shows only the icon.
- [x] Clicking opens the menu below the button as in `07a`: width 184, `radius::LG`, `color::OVERLAY`, `shadow_overlay`, 4 px inset, rows `size::MENU_ROW` (32) with labels "Off", "3 seconds", "10 seconds", a `check` icon on the current value, row hover `ITEM_HOVER`. Opens/closes with a 120 ms opacity fade (`motion::MENU`).
- [x] Choosing a row sets the Timer and closes the menu. Clicking outside, or pressing Esc, closes it without changing the Timer.
- [x] T cycles Off → 3s → 10s → Off (core event from ticket 02) and updates the button; if the menu is open it also updates the check.
- [x] Core tests: cycling, selecting, closing without change.

**Out of this ticket:** Countdown (10), tooltip "Timer · T" (16), locking (12).

## Verification

- `cargo test timer`: 4 passed. New selection and dismissal tests were observed failing before their core events were implemented.
- `cargo test`: 58 passed, including GStreamer integration tests.
- `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`: passed.
- Native GPUI/Wayland check using a temporary source copy with camera detection replaced by `videotestsrc` (no physical webcam available): Off/3s/10s selection, live T cycling/check updates, Esc/outside dismissal, button toggling, row hover and menu fades. The production backend is unchanged. Screenshots: [Off](../validation/09-timer-setting/off.png), [3s menu](../validation/09-timer-setting/menu-3s.png), [closed 3s](../validation/09-timer-setting/closed-3s.png).
- Standards and spec reviews completed; all findings resolved.

## Implementation notes
