# 19: Error screens and Retry

**What to build:** When the camera cannot be used the app shows one of three distinct error screens with a Retry button that re-detects the camera without restarting.

**Blocked by:** 03

**Status:** ready-for-agent

**Design reference:** `html/06-error-busy.html`, `html/06-error-denied.html`, `html/06-error-missing.html` and their PNGs.

- [ ] Layout for all three, as in the design HTML: background `color::APP_BG`; camera name top left (`camera` icon + name, `TEXT_TERTIARY`, or "No camera" for the missing case); centered column with a 56 px tile (`radius::FULL`, `ICON_TILE` fill, 1 px `border_subtle`, icon `icon.xl` in `TEXT_SECONDARY`: `camera-off`), gap `space.24`, title (`type.title`), message (`type.body`, `TEXT_SECONDARY`, max width 380), then a primary Retry button (`size 36` tall, `radius::FULL`, `color::SELECTED` fill, `TEXT_ON_SELECTED`, `type.label` 600, `rotate-cw` icon `icon.md` `stroke.bold` + "Retry").
- [x] Texts, exactly: **InUse** title "Camera is in use", message "Another app is using {camera name}. Close it, then try again." **AccessDenied** title "Camera access denied", message "Camera doesn’t have permission to open the webcam. Check the device permissions, then try again." **NoCamera** title "No camera found", message "Connect a webcam, then try again."
- [ ] Classification comes from errors/events of the capture stack (GStreamer bus messages, V4L2 errors such as EBUSY / EACCES / device not found). The real backend maps them to the `InUse`, `AccessDenied` and `NoCamera` kinds from ticket 02. Any other failure is **not** invented a screen for: ask the user which screen to show.
- [x] Retry re-runs camera detection and Viewfinder start. On success the app shows the live Viewfinder; on failure it shows the matching error screen again. No app restart needed.
- [x] Core tests with the fake backend: each kind shows its screen; Retry moves to the ready state on success; Retry after another failure shows the new kind.

**Out of this ticket:** Disconnection during use (20).

## Decision

The user approved the same layout for other capture failures, with an appropriate icon and friendly message. General failures now use an alert triangle, “Unable to start camera”, and guidance to retry or check installed camera/media components.

## Implementation notes

- **Non-blocking, medium urgency — graphical verification:** The three layouts are implemented in `src/error_ui.rs:10` and checked against `docs/design/Camera-UI/html/06-error-*.html` by code review, but have not been visually verified in a running GPUI window. Check all three error screens and Retry with a real camera. `cargo check --offline`, `cargo clippy --offline --all-targets -- -D warnings`, and `cargo test --offline` passed (93 tests); these do not establish pixel-level layout correctness.

- **Non-blocking, medium urgency — physical classification:** Known resource errors and unknown/missing-plugin errors have automated classifier coverage; real V4L2 busy/permission cases still require a camera.
