# Camera App MVP

Status: ready-for-agent

## Problem Statement

I want to take Photos and record Videos with my computer's webcam on a Hyprland desktop (a tiling Wayland compositor), using a lightweight, keyboard-first app. I have no simple camera app that behaves like a phone camera: a full-window Viewfinder, a single Shutter, a Timer and a Mirror, with no gallery or filters, and that does not lose my recordings when something fails (camera busy, camera disconnected, window closed).

## Solution

A desktop app in Rust (GPUI) whose Viewfinder fills the window, with overlaid controls that fade out after a few seconds without mouse movement. The user picks a Mode (Photo | Video), presses the Shutter (optionally after a 3s or 10s Timer), and the Capture is saved to disk. The Last-Capture Thumbnail shows the latest Capture and opens its folder in the file manager. Camera failures appear as distinct error screens with Retry, and no Video is considered saved until its file is finalized.

The interface follows `docs/design/Camera-UI` (7 screens, component sheet and tokens), with one adjustment: a `disabled` state for locked controls.

## User Stories

1. As a user, I want the Viewfinder to fill the window with letterbox bars when the aspect ratio differs, so that I can frame a Capture accurately at any window size
2. As a user, I want the app to pick my webcam automatically, so that I can start using it without configuring anything
3. As a user, I want to see the active camera name discreetly in the top left, so that I know which device is in use
4. As a user, I want to take a Photo with a single press of the Shutter, so that I can capture a moment quickly
5. As a user, I want a brief white flash after a Photo, so that I know it was taken
6. As a user, I want Photos saved as JPEG in my Pictures folder under a Camera subfolder, so that they open in any program
7. As a user, I want Photos named by date and time (`YYYY-MM-DD_HH-MM-SS`), so that they sort chronologically and never collide
8. As a user, I want to switch between Photo and Video Mode with two separate buttons in a pill above the Shutter, so that I can see and choose the Mode in one click
9. As a user, I want the active Mode highlighted in the pill, so that I know what the next Shutter press will do
10. As a user, I want to start and stop a Video with the same Shutter, so that recording is simple
11. As a user, I want the Shutter to turn into a red square while recording, so that it is obvious I am recording
12. As a user, I want a red dot and the elapsed time visible at all times while recording, so that I always know how long I have recorded
13. As a user, I want Videos saved as MP4 (H.264 + AAC) in my Videos folder under a Camera subfolder, so that they play anywhere
14. As a user, I want my Video to include audio from the default microphone, so that my recordings have sound
15. As a user, I want the Video to be recorded without audio, without error, when no microphone exists, so that a missing mic never blocks recording
16. As a user, I want a Timer with off, 3s and 10s, so that I can get into frame before the Capture starts
17. As a user, I want a big countdown number over the Viewfinder, so that I can see the time left
18. As a user, I want the Timer to apply to Videos as well, so that recording begins after the countdown
19. As a user, I want to cancel a countdown with Esc or by pressing the Shutter (shown as ✕), so that I can abort a mistaken Timer
20. As a user, I want an explicit "Esc to cancel" hint during the countdown, so that I know how to abort
21. As a user, I want Mode, Timer, Resolution and Mirror dimmed with a tooltip explaining why while a countdown or recording is active, so that I understand why they cannot be changed
22. As a user, I want the Mirror toggle to flip the Viewfinder horizontally, so that it behaves like a mirror when I frame myself
23. As a user, I want Mirror on by default, so that framing feels natural
24. As a user, I want the saved Photo or Video never to be mirrored, so that text and scenes are not reversed
25. As a user, I want a Resolution menu listing only what my camera actually supports, so that I never pick an unsupported size
26. As a user, I want the highest 16:9 Resolution selected on startup, so that I get the best quality by default
27. As a user, I want to change Resolution between Captures and have the Viewfinder restart at the new size, so that Captures use the size I chose
28. As a user, I want the Last-Capture Thumbnail at the bottom left, so that I can see my latest Capture like on a phone
29. As a user, I want the Thumbnail of a Video to show a frame with a small play icon, so that I can tell it is a Video
30. As a user, I want clicking the Thumbnail to open the Captures folder in my file manager, so that I can browse my Captures without an in-app gallery
31. As a user, I want the Thumbnail to reflect the most recent Capture on disk when the app starts, so that it is accurate across sessions
32. As a user, I want an empty placeholder Thumbnail (still opening the folder) when I have no Captures, so that the layout stays stable
33. As a user, I want the Captures folders created automatically on first Capture, so that I do not need to set them up
34. As a user, I want Space to press the Shutter, P/V to select Photo/Video, T to cycle the Timer, M to toggle Mirror and Esc to cancel a countdown, so that I can use the app without the mouse
35. As a user, I want these shortcuts to work even when the controls have faded out, so that hidden controls never disable the keyboard
36. As a user, I want tooltips on hover showing each control's shortcut, so that I can learn them without a help screen
37. As a user, I want the controls to fade out after a few seconds without mouse movement and return on movement, so that the Viewfinder stays clean
38. As a user, I want the controls to stay visible during a countdown and during recording, so that I never lose the cancel/stop control
39. As a user, I want the Shutter to show "Saving…" and be disabled after I stop a Video until the file is finalized, so that I do not start another Capture mid-save
40. As a user, I want the Thumbnail to update, and the Video to be considered saved, only after the file is finalized successfully, so that the app never claims a Video exists when it does not
41. As a user, I want a brief fading error notice when a Capture cannot be written (disk full, permissions) and no Thumbnail update, so that I know the Capture was not saved
42. As a user, I want closing the window during a recording to finalize the Video and then exit, so that I never end up with a corrupt MP4
43. As a user, I want a distinct "camera in use" screen with Retry when another app holds the camera, so that I know to close it and try again
44. As a user, I want a distinct "access denied" screen with Retry when I lack permission, so that I can fix permissions and try again
45. As a user, I want a distinct "no camera" screen with Retry when no camera exists, so that I can plug one in and try again
46. As a user, I want Retry to re-detect cameras and re-initialize without restarting the app, so that recovery is one click
47. As a user, I want a camera disconnection during recording to stop the recording cleanly, save the partial Video when technically possible, and then show the matching error screen, so that I lose as little as possible
48. As a user, I want the app to start in Photo Mode with Timer off, Mirror on and the highest 16:9 Resolution every time, so that behavior is predictable without a config file
49. As a user, I want the window to be a normal resizable floating window with no custom title bar, so that my compositor manages it
50. As a developer, I want the camera backend behind a trait with a fake implementation, so that flows and failures can be tested without hardware
51. As a developer, I want the video codec/container and the audio source behind traits, so that swapping `openh264enc` for another encoder does not touch the core

## Implementation Decisions

- **UI toolkit:** GPUI (ADR 0001). Visuals follow `docs/design/Camera-UI`: dark theme, one red accent, flat/semi-transparent fills, rounded rectangles, SVG icons, opacity fades only, no backdrop blur. Add a `disabled` state (reduced opacity + tooltip) to the component sheet and tokens.
- **Camera/recording stack:** GStreamer (ADR 0002). One pipeline from the V4L2 source feeds the Viewfinder (frames delivered to GPUI through an `appsink`, since no GTK paintable sink is available), Photo capture and Video recording.
- **Core module (the main seam):** a pure state machine with no GStreamer or GPUI dependencies. It owns Mode, Timer setting, Mirror, Resolution selection, countdown, recording, finalizing and error states, and which controls are locked. It consumes input events (Shutter, shortcuts, menu selections, Retry, window close) and backend events, and emits commands to the backend and effects for the UI (flash, notices, thumbnail refresh).
- **Camera backend trait:** operations to detect cameras and capabilities, start/stop the Viewfinder at a Resolution, take a Photo, start/stop a Video, and finalize on close. It reports events: camera ready, errors classified as in use / access denied / no camera / disconnected, write failure, and EOS (finalization complete). A real GStreamer implementation and a fake implementation exist.
- **Codec and audio traits:** the Video encoder/muxer and the audio source are separate traits. First implementations: `openh264enc` + `avenc_aac` in MP4 (`x264enc` is not installed on the target machine), and `pulsesrc` default device for audio. Missing microphone yields a silent Video.
- **Error classification:** derived from capture-system events (V4L2, PipeWire, GStreamer messages), not solely from failing to open a device node. Retry re-runs camera detection and initialization.
- **Photo:** taken from the live Viewfinder stream at the active Resolution as JPEG (the camera delivers MJPG). Mirror is never applied to saved files.
- **Cameras and formats:** the first capture-capable video device is chosen automatically (metadata nodes ignored); only MJPG modes are offered, since YUYV tops out at 640x480 on the reference camera. The Resolution menu is built from the camera's reported capabilities; initial selection is the highest 16:9 mode.
- **Output:** Photos to the XDG Pictures directory under `Camera`, Videos to the XDG Videos directory under `Camera`, named `YYYY-MM-DD_HH-MM-SS` plus extension; folders created on first Capture.
- **Last-Capture Thumbnail:** computed at startup by finding the newest Capture in both folders and refreshed after each confirmed Capture. Video thumbnails are a frame extracted by a separate pipeline plus a play icon. Activating it opens the folder via the system opener.
- **Recording lifecycle:** stopping a Video enters a "Saving…" state with the Shutter disabled until EOS is received; only then is the Capture considered saved. Closing the window or losing the camera during recording triggers an orderly EOS before exit or before showing the error screen. Write failures surface a fading notice and do not update the Thumbnail.
- **Lock rules:** during countdown and recording, Mode, Timer, Resolution and Mirror are disabled and visibly dimmed with an explanatory tooltip. During countdown the Shutter acts as cancel (✕), same as Esc.
- **Controls visibility:** controls fade after a few seconds without mouse movement, but stay visible during countdown and recording. Shortcuts (Space, P, V, T, M, Esc) work regardless of visibility.
- **Settings:** nothing persisted. Startup defaults: Photo, Timer off, Mirror on, highest 16:9 Resolution.
- **Glossary:** use the terms in `GLOSSARY.md` (Viewfinder, Capture, Photo, Video, Mode, Shutter, Timer, Mirror, Resolution, Last-Capture Thumbnail) consistently in code and UI.

## Testing Decisions

- **What makes a good test:** exercises external behavior only: feed input and backend events to the core, assert the resulting state and emitted backend commands/effects. No assertions on internal fields or call order that users cannot observe.
- **Main seam:** the core state machine driven against the fake camera backend. Fake simulates busy, denied, missing, disconnection mid-recording, write failure, missing microphone and EOS timing. Covers: Mode switching, Timer countdown and cancel, Shutter-as-cancel, lock rules, Photo flow, Video start/stop, "Saving…" until EOS, close-while-recording, Retry, Thumbnail refresh only after confirmed save, startup defaults, Resolution list and default.
- **Secondary seam (optional integration):** the real GStreamer backend against `videotestsrc`/`audiotestsrc`, asserting that a valid JPEG and a playable MP4 with finalized container are produced. Skipped where plugins are missing.
- **Not tested:** GPUI rendering and visual layout; verified manually against the design PNGs.
- **Prior art:** none; the repo has only an empty `main.rs`. Written test-first with `/tdd`.

## Out of Scope

- Filters, zoom, manual exposure, camera selector, in-app gallery.
- Persisting settings between runs.
- Multiple cameras at once, hot-switching cameras.
- Hardware encoding and codecs other than the first `openh264enc` + AAC implementation.
- Windows/macOS or X11-specific work; target is Linux/Wayland (Hyprland).
- Custom title bar and light theme.

## Further Notes

- Reference machine: Integrated Webcam HD (MJPG 1280x720, 960x540, 848x480, 640x480, 640x360), GStreamer 1.28 with `v4l2src`, `pipewiresrc`, `pulsesrc`, `mp4mux`, `openh264enc`, `avenc_aac`, `jpegdec`; `x264enc` and `gtk4paintablesink` are absent.
- Design assets: `docs/design/Camera-UI` (PNG and standalone HTML for each screen, plus component sheet and tokens). Design prompt in `.scratch/ui-design/claude-design-prompt.md`.
- ADRs: `docs/adr/0001-gpui-for-ui.md`, `docs/adr/0002-gstreamer-for-camera-and-recording.md`.
- Next step: `/mattpocock-skills:to-tickets` to cut vertical slices (Viewfinder → Photo → Timer → Video → Mirror/Resolution → errors → Thumbnail).

## Ground rules for implementing agents

These apply to every ticket in `issues/`. Tickets link here instead of repeating them.

1. **Authority order:** the ticket, then this spec, then `GLOSSARY.md`, then the ADRs in `docs/adr/`, then the design files. If two of them conflict, stop and report the conflict. Do not pick one silently.
2. **Unspecified means ask.** If a behaviour, text, size, color, duration, default or edge case is not written in the ticket, this spec or the design, stop and ask the user. Do not invent features, options, settings, config files, logging, telemetry or UI that no ticket lists.
3. **UI fidelity:** every color, size, radius, spacing, font size/weight, icon size and duration comes from the design tokens (`docs/design/Camera-UI/html/09-design-tokens.html`, the `theme.rs` block, ported verbatim by ticket 01). No literal values in UI code. Positions and layout come from the standalone HTML of the screen named in the ticket (`docs/design/Camera-UI/html/`), and the result is compared against its PNG (`docs/design/Camera-UI/png/`). The design uses sample data ("Integrated Camera", "1080p"); the app shows real data.
4. **Vocabulary:** use the `GLOSSARY.md` terms in code identifiers, UI text and docs. Do not use the terms it lists under _Avoid_.
5. **Code docs:** per `CLAUDE.md`, every fn/method has a `///` comment saying what it does.
6. **Tests:** behavior of the core is tested through the main seam (see Testing Decisions), written test-first. No tests of GPUI rendering. Where a ticket says "integration test", it runs the real GStreamer backend against `videotestsrc`/`audiotestsrc` and is skipped (not failed) when a required plugin is missing.
7. **Done means:** every acceptance criterion in the ticket is checked, `cargo build` and `cargo test` pass, and no new compiler warnings.
8. **Scope:** do only what the ticket lists. Behavior owned by a later ticket stays unimplemented; where the ticket says what to show in the meantime, show exactly that.
9. **Reference machine facts** (Integrated Webcam HD, installed GStreamer plugins) are in Further Notes; do not assume plugins beyond those listed there.
