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
- [ ] The latest frame is drawn filling the window while keeping its aspect ratio; unused window area is `color::LETTERBOX`. Resizing the window keeps the image centered and undistorted.
- [ ] Top-left: the `camera` icon (`icon.sm`) and the camera name in `type.small`, `color::TEXT_SECONDARY`, positioned as in `01-ready-photo.html` (inset `space.16`, 8 px gap). Behind it, the `scrim_top` gradient (96 px, black at `color::SCRIM_TOP` fading to transparent).
- [x] No other control is drawn in this ticket.
- [ ] If detection or opening fails, the window shows `color::APP_BG` and nothing else. The failure is not handled further (ticket 19).
- [x] Integration test: the real backend, pointed at `videotestsrc`, delivers frames to the appsink at the requested Resolution.
- [ ] Manual check, stated in the PR description: live video on the reference machine at 1280×720.

**Out of this ticket:** Mirror, Resolution menu, error screens, auto-hide of the camera name.

## Implementation notes

- **Letterbox, centering and resize (unchecked).** Attempted: the frame is drawn with `img(..).size_full().object_fit(ObjectFit::Contain)` on a `color::LETTERBOX` root (`src/camera_window.rs`, `viewfinder_image` and `Render::render`). GPUI's `ObjectFit::Contain` keeps the aspect ratio and centers the image (gpui-0.2.2 `src/style.rs`). What happened: the agent ran the app on the reference machine but could not look at the window, so the result has not been checked by eye. Non-blocking, urgency low. Remaining: a human resizes the window and checks the image stays centered and undistorted.
- **Top-left camera name and `scrim_top` (unchecked).** Attempted: `camera_label` and `scrim_top` in `src/camera_window.rs` use the 01-ready-photo.html values (inset `space::S16`, row height `size::CONTROL`, gap `space::S8`, `font::SMALL` with `FontWeight::MEDIUM`, `TEXT_SECONDARY`, 96 px gradient at 180° from `SCRIM_TOP` to transparent). What happened: not compared visually against `png/01-ready-photo.png`. Non-blocking, urgency low. Remaining: a visual check. The name shown is the raw V4L2 card name (`Integrated_Webcam_HD: Integrate` on the reference machine), as the ticket asks.
- **Failure shows only `APP_BG` (unchecked).** Attempted: any non-`Ready` camera state renders only the `APP_BG` root (`src/camera_window.rs`, `Render::render`). Detection or start failures are mapped to `CameraError` (`src/gst_camera.rs`, `play` and `camera_error`). A camera with no MJPG mode, or with only range-sized MJPG modes, is reported as `NoCamera`. What happened: checked by reading the code only. No busy, denied or missing camera was tried at runtime. Non-blocking, urgency low (ticket 19 covers this). Remaining: a run with the camera busy or unplugged.
- **Manual check at 1280×720 (unchecked).** Attempted: `GST_DEBUG=v4l2src:5 ./target/debug/my-own-camera-app-rust` showed v4l2src negotiating `image/jpeg, width=1280, height=720, framerate=30/1`, with buffers flowing and no crash over 15 to 30 s runs. What happened: no one has watched the live video yet. Non-blocking, urgency medium. Remaining: a human confirms live video on screen and notes it in the PR description.
- **GPU crash fixed during the spike (for reference).** Freeing each replaced frame's texture straight away with `cx.drop_image` crashed the Vulkan renderer (`GPU has crashed` in blade-graphics-0.7.1 `src/vulkan/command.rs:441`). The texture is now freed one frame late (`previous_frame` in `src/camera_window.rs`, `CameraWindow::show`). Three 15 s runs and one 30 s run did not crash. Non-blocking, urgency low: watch for it in longer sessions.
