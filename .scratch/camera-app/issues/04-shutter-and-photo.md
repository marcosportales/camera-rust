# 04: Shutter and Photo

**What to build:** The Shutter button and Space key take a Photo and save it as a JPEG, with the flash and the "Photo saved" toast.

**Blocked by:** 03

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html` and `html/05-photo-flash.html` (`png/01-ready-photo.png`, `png/05-photo-flash.png`), component sheet `html/08-components.html` (Shutter states).

**GPUI references:** Key binding (Space): [`input.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/input.rs), [`key_dispatch.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md); Flash and toast opacity: [`opacity.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/opacity.rs), [`animation.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/animation.rs).

- [ ] Shutter: bottom center, `size::SHUTTER` (72) total, ring `size::SHUTTER_RING` (4) in `color::SHUTTER_RING`, 4 px gap, fill `size::SHUTTER_FILL` (56). Fill is `SHUTTER_FILL`; hover `SHUTTER_FILL_HOVER`; pressed `SHUTTER_FILL_PRESSED`; fades 100 ms (`motion::HOVER`). Bottom inset `space.24`. Behind it, the `scrim_bottom` gradient (200 px).
- [x] Clicking the Shutter, or pressing Space, takes one Photo. Every press takes one Photo: no debounce.
- [x] The Photo is the current Viewfinder frame at the active Resolution, saved as a valid JPEG. The MJPG frame is not re-encoded. Mirror never applies (it does not exist yet).
- [x] Saved to the XDG Pictures directory plus `Camera` (falls back to `~/Pictures/Camera` if XDG is unavailable). The folder is created when missing.
- [x] File name is local time `YYYY-MM-DD_HH-MM-SS.jpg`. If the name exists, append `_1`, `_2`, … before the extension, never overwriting.
- [ ] Flash: a full-window `color::FLASH` overlay drawn **under** the controls, fading in over `motion::FLASH_IN` (40 ms) to its peak and out over `motion::FLASH_OUT` (260 ms). While it plays the Shutter fill is `SHUTTER_FILL_PRESSED`.
- [x] Toast, top center as in `05-photo-flash.html`: `size::STATUS` (32) tall, `radius::FULL`, `color::OVERLAY` fill, `shadow_overlay`, `check` icon (`icon.md`, `stroke.bold`), text "Photo saved" (`type.small`, `TEXT_PRIMARY`) followed by the folder relative to the home directory (e.g. `Pictures/Camera`, `TEXT_TERTIARY`). Fades in 150 ms, holds 2000 ms, fades out 300 ms (`motion::TOAST_HOLD`).
- [ ] The toast and flash appear only after the file is fully written.
- [x] If the write fails, no flash and no success toast; the same toast shape appears with the `x` icon and text "Couldn't save photo". Nothing else changes.
- [x] The core gets a "take photo" command and "photo saved / photo failed" events; core tests cover success and failure through the fake backend, which is extended with a scriptable Photo result.
- [x] Integration test: the real backend writes a valid JPEG from `videotestsrc` to a temp directory.

**Out of this ticket:** Mode pill (Photo is the only Mode usable now), Timer, Thumbnail, Mirror, tooltips.

## Implementation notes

Manually verified by the user with a real webcam: Shutter display, hover, click and Space (including rapid presses: 7 files within two seconds, `_1`…`_5` suffixes, nothing overwritten), JPEG output, the "Photo saved" toast and the "Couldn't save photo" toast with the `x` icon (folder made read-only). Remaining items were implemented but not confirmed; all are non-blocking, urgency low to medium.

- **Flash** (`src/camera_window.rs` `render`, `src/photo_ui.rs` `flash`): the user could not tell whether it plays. It lasts 300 ms at 80% white, so it may be subtle. Whether it is drawn under the controls and the Shutter shows the pressed tone during it are unconfirmed; `envelope` timing is unit-tested. Also unconfirmed: that a failed save shows no flash (nothing visible to compare against).
- **`scrim_bottom` gradient** (`src/camera_window.rs` `scrim_bottom`): the user could not tell. Gradient angle 0 is assumed to fade upward from the bottom edge. Compare with `docs/design/Camera-UI/png/01-ready-photo.png` to confirm.
- **Shutter pressed state and 100 ms fade** (`src/photo_ui.rs` `FillTransition`): hover confirmed; press and fade duration not. Known quirk: if the fill target changes mid-fade, the colour jumps to the previous target before fading again (low).
- A slow disk could stall the Viewfinder because the write is synchronous on the UI thread (`take_photo` in `src/camera_window.rs`) (low).
