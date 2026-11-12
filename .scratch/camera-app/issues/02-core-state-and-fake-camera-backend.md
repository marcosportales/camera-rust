# 02: Core state and fake camera backend

**What to build:** The pure core (no GStreamer, no GPUI) that holds the app's state and reacts to events, plus the camera backend trait and its fake implementation. This is the main test seam of the project. Only tests exercise it in this ticket.

**Blocked by:** 01

**Status:** ready-for-agent

**Design reference:** none (no UI).

- [ ] The core crate/module compiles without depending on GStreamer or GPUI.
- [ ] Startup state: Mode = Photo, Timer = Off, Mirror = on, no Resolution selected yet, camera state = "detecting".
- [ ] A `Resolution` is a width × height in pixels. A Resolution is **16:9** when `width * 9` is within 1% of `height * 16` (so 1280×720, 960×540, 848×480 and 640×360 are 16:9; 640×480 is not).
- [ ] Default Resolution rule, implemented as one function: the highest-pixel-count 16:9 Resolution the camera reports; if the camera reports no 16:9 Resolution, the highest-pixel-count Resolution overall.
- [ ] The camera backend trait has exactly these operations in this ticket: detect cameras (returns the first capture-capable camera: its name and its list of supported MJPG Resolutions) and start the Viewfinder at a given Resolution. Later tickets extend the trait; do not add operations now.
- [ ] The trait reports events to the core: camera ready (name, Resolutions) and camera error of exactly one of three kinds: `InUse`, `AccessDenied`, `NoCamera`.
- [ ] The fake backend can be scripted to return each of: a ready camera with a chosen name and Resolution list, and each of the three error kinds.
- [ ] Tests (through the core's public interface only) cover: startup state; default Resolution for a list with several 16:9 entries; for a list with no 16:9 entry; each error kind moving the core to a distinct error state; a ready camera moving the core to the ready state with the default Resolution selected.
- [ ] Mode, Timer and Mirror have change events defined in the core (set Mode, cycle Timer Off → 3s → 10s → Off, toggle Mirror) with tests for each, including Timer wrapping from 10s back to Off.

**Out of this ticket:** Photo, Video, countdown, recording, Retry behavior (ticket 19), any UI.
