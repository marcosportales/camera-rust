# 17: Last-Capture Thumbnail (Photos)

**What to build:** The Thumbnail at the bottom left shows the most recent Capture and opens the Captures folder. This ticket covers Photos and the empty state.

**Blocked by:** 04

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/08-components.html` (Last-capture thumbnail: empty, photo, hover, just saved).

**GPUI references:** Image display: [`image_gallery.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/image_gallery.rs).

- [ ] Thumbnail left of the Shutter at `space.40` gap, `size::THUMB` (48) square, `radius::MD`, images cropped to fill. Hover/"just saved" looks as in the component sheet.
- [ ] At startup the app finds the newest file (by modification time) among `*.jpg` in the Photos folder and `*.mp4` in the Videos folder, and shows it. In this ticket only a `*.jpg` can be shown; if the newest file is an `.mp4`, the empty look is shown (ticket 18 handles it).
- [ ] If neither folder exists or both are empty, the empty look from the component sheet (the `folder` icon at `icon.md`) is shown, and the Thumbnail remains clickable.
- [ ] After a Photo is saved (confirmed per ticket 04), the Thumbnail updates to it with the "just saved" look from the component sheet.
- [ ] Clicking opens the folder of the displayed Capture in the system file manager through the system opener (`xdg-open`). In the empty look it opens the Photos folder if it exists, otherwise nothing happens and no error is shown.
- [ ] Core/IO tests: newest-file selection over a temp tree (jpg newer than mp4, empty folders, missing folders).

**Out of this ticket:** Video thumbnails (18), tooltip (16).
