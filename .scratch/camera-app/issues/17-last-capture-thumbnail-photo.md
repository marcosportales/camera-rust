# 17: Last-Capture Thumbnail (Photos)

**What to build:** The Thumbnail at the bottom left shows the most recent Capture and opens the Captures folder. This ticket covers Photos and the empty state.

**Blocked by:** 04

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html`, `html/08-components.html` (Last-capture thumbnail: empty, photo, hover, just saved).

**GPUI references:** Image display: [`image_gallery.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/image_gallery.rs).

- [x] Thumbnail left of the Shutter at `space.40` gap, `size::THUMB` (48) square, `radius::MD`, images cropped to fill. Hover/"just saved" looks as in the component sheet.
- [x] At startup the app finds the newest file (by modification time) among `*.jpg` in the Photos folder and `*.mp4` in the Videos folder, and shows it. In this ticket only a `*.jpg` can be shown; if the newest file is an `.mp4`, the empty look is shown (ticket 18 handles it).
- [x] If neither folder exists or both are empty, the empty look from the component sheet (the `folder` icon at `icon.md`) is shown, and the Thumbnail remains clickable.
- [x] After a Photo is saved (confirmed per ticket 04), the Thumbnail updates to it with the "just saved" look from the component sheet.
- [ ] Clicking opens the folder of the displayed Capture in the system file manager through the system opener (`xdg-open`). In the empty look it opens the Photos folder if it exists, otherwise nothing happens and no error is shown.
- [x] Core/IO tests: newest-file selection over a temp tree (jpg newer than mp4, empty folders, missing folders).

**Out of this ticket:** Video thumbnails (18), tooltip (16).

## Implementation notes

- Folder opening is implemented and target selection is verified by `cargo test capture_store::tests` (9 passing tests), including the missing-Photos no-op. End-to-end clicking through the live app into the desktop file manager remains unverified: `ls /dev/video*` reports `No such file or directory`, so the live app cannot expose its camera-ready controls here. Non-blocking, urgency low. Manually click a displayed Photo and the empty Thumbnail with Photos present/absent on a machine with a webcam. References: `src/camera_window/thumbnail.rs:14`, `src/capture_store.rs:64`. Native empty/Photo/just-saved/hover visuals were inspected in a temporary GPUI window; the harness was removed. `cargo test` passed all 91 tests; `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` passed.
