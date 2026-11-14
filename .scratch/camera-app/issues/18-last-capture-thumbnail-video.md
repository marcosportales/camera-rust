# 18: Last-Capture Thumbnail (Videos)

**What to build:** Videos appear in the Thumbnail as a frame with a play badge.

**Blocked by:** 07, 17

**Status:** ready-for-agent

**Design reference:** `html/08-components.html` (Last-capture thumbnail "video"), token `thumb_badge`.

- [x] The Thumbnail frame of a Video is extracted by a separate GStreamer pipeline from the file (a frame near the start). It does not touch the live recording pipeline.
- [ ] A Video Thumbnail shows an 18 px circular badge, `color::THUMB_BADGE` fill, with the `play` icon (`icon.play`, 10 px, filled), positioned as in the component sheet.
- [x] At startup, if the newest file among both folders is an `.mp4`, its frame and badge are shown. If extraction fails, the empty look is shown.
- [x] After a Video is saved and confirmed by EOS (ticket 07), the Thumbnail updates with the "just saved" look. It never updates before EOS confirmation or when saving failed.
- [x] Clicking opens the Videos folder for a Video Thumbnail, via `xdg-open`.
- [x] Integration test: frame extraction from the MP4 produced by ticket 06's test returns an image.

**Out of this ticket:** Anything else about the folder or gallery.

## Implementation notes

- Badge visual verification (non-blocking, low urgency): implemented the 18 px circle, `THUMB_BADGE` fill, 10 px filled play icon, and 3 px bottom/right positioning in `src/thumbnail_ui/video.rs:8` and `assets/icons/play.svg:1`. Code and Clippy checks pass. The app launched using `XDG_CONFIG_HOME=/tmp/camera-ticket18/config target/debug/my-own-camera-app-rust`, but automatic approval review rejected `grim /tmp/camera-ticket18/startup.png` because a full-desktop screenshot could capture unrelated sensitive content. The checkbox remains open pending visual inspection of the rendered badge against `docs/design/Camera-UI/html/08-components.html:120`.
