# 09: Timer setting (button, menu, T shortcut)

**What to build:** The Timer button opens a menu to choose Off / 3 seconds / 10 seconds, and T cycles the setting. The countdown itself comes later; this ticket only sets and shows the Timer.

**Blocked by:** 05

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/07a-timer-menu.html` / `png/07a-timer-menu.png`, `html/08-components.html` (Icon buttons, Menu).

- [ ] Timer button, top right (inset `space.16`), `size::CONTROL` (40) square, `radius::MD`, `color::CONTROL_HOVER` fill as in `01-ready-photo.html`, `shadow_control`, `timer` icon (`icon.lg`). Layout order with future top-right controls follows the design: Timer button, then Resolution button, 8 px apart (`space.8`).
- [ ] When the Timer is 3s or 10s, the button shows its "set" look from `07a-timer-menu.html` (`color::CONTROL_ON` fill with the value text "3s"/"10s"). When Off it shows only the icon.
- [ ] Clicking opens the menu below the button as in `07a`: width 184, `radius::LG`, `color::OVERLAY`, `shadow_overlay`, 4 px inset, rows `size::MENU_ROW` (32) with labels "Off", "3 seconds", "10 seconds", a `check` icon on the current value, row hover `ITEM_HOVER`. Opens/closes with a 120 ms opacity fade (`motion::MENU`).
- [ ] Choosing a row sets the Timer and closes the menu. Clicking outside, or pressing Esc, closes it without changing the Timer.
- [ ] T cycles Off → 3s → 10s → Off (core event from ticket 02) and updates the button; if the menu is open it also updates the check.
- [ ] Core tests: cycling, selecting, closing without change.

**Out of this ticket:** Countdown (10), tooltip "Timer · T" (16), locking (12).
