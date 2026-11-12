# 03: Live Viewfinder

**What to build:** The real GStreamer camera backend and the Viewfinder: the live image fills the window, with the camera name top left. Opening the app with a webcam connected shows live video.

**Blocked by:** 01, 02

**Status:** ready-for-agent

**Design reference:** `html/01-ready-photo.html` / `png/01-ready-photo.png`, only these parts: image, `scrim_top` gradient, camera name. ADR `0002`.

**GPUI references:** Async from entities: [`contexts.md`](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md); Images: [`image_loading.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/image_loading.rs) (`RenderImage`); `Window::paint_image` in `window.rs`. No example feeds live camera frames: expect to spike the GStreamer-to-GPUI frame handoff here.

- [x] A real implementation of the camera backend trait from ticket 02 uses GStreamer: `v4l2src` requesting MJPG, decoded and delivered to the app through an `appsink`.
- [x] Camera selection: the first `/dev/video*` node with video-capture capability, skipping metadata nodes. The camera name is the device name reported by V4L2 (not hard-coded).
- [x] Supported Resolutions are read from the camera's MJPG modes only. YUYV modes are ignored.
- [x] The Viewfinder starts at the default Resolution (rule in ticket 02), with the camera's native frame rate. No frame rate is forced.
- [x] The latest frame is drawn filling the window while keeping its aspect ratio; unused window area is `color::LETTERBOX`. Resizing the window keeps the image centered and undistorted.
- [x] Top-left: the `camera` icon (`icon.sm`) and the camera name in `type.small`, `color::TEXT_SECONDARY`, positioned as in `01-ready-photo.html` (inset `space.16`, 8 px gap). Behind it, the `scrim_top` gradient (96 px, black at `color::SCRIM_TOP` fading to transparent).
- [x] No other control is drawn in this ticket.
- [ ] If detection or opening fails, the window shows `color::APP_BG` and nothing else. The failure is not handled further (ticket 19).
- [x] Integration test: the real backend, pointed at `videotestsrc`, delivers frames to the appsink at the requested Resolution.
- [x] Manual check, stated in the PR description: live video on the reference machine at 1280×720.

**Out of this ticket:** Mirror, Resolution menu, error screens, auto-hide of the camera name.

## Implementation notes

- **Failure shows only `APP_BG` (unchecked).** Attempted: any non-`Ready` camera state renders only the `APP_BG` root (`src/camera_window.rs`, `Render::render`). Detection or start failures are mapped to `CameraError` (`src/gst_camera.rs`, `play` and `camera_error`). A camera with no MJPG mode, or with only range-sized MJPG modes, is reported as `NoCamera`. What happened: checked by reading the code only. No busy, denied or missing camera was tried at runtime. Non-blocking, urgency low (ticket 19 covers this). Remaining: a run with the camera busy or unplugged.
