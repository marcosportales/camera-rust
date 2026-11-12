# 01: App skeleton and design tokens

**What to build:** A GPUI app that opens an empty Camera window with the project's design tokens, the Inter font and the SVG icon set available to every later ticket. No Viewfinder, no controls.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Design reference:** `docs/design/Camera-UI/html/09-design-tokens.html` (the `theme.rs` block), `docs/design/Camera-UI/html/fonts/`.

- [ ] `cargo run` opens one window titled `Camera`, default size 960 × 540 logical px, resizable, with no custom title bar drawn by the app.
- [ ] The window background is `color::APP_BG`.
- [ ] Closing the window ends the process with exit code 0.
- [ ] A theme module contains the `theme.rs` block from the design tokens page **verbatim**: `color`, `radius`, `space`, `size`, `font`, `motion` modules with every constant and value listed there. Nothing added, renamed or removed.
- [ ] Inter weights 300, 400, 500 and 600 from the design's `fonts/` folder are bundled and registered, and `font::FAMILY` resolves to them. No other font is bundled.
- [ ] The Lucide icons `timer`, `flip-horizontal`, `camera`, `camera-off`, `check`, `x`, `rotate-cw`, `play`, `folder` and `chevron-down` are bundled as SVG assets and loadable by name through GPUI's asset source. Stroke width and sizes are applied by the code that draws them, not baked into the files.
- [ ] A unit test asserts every icon name above resolves to non-empty SVG data.
- [ ] A unit test asserts `motion::IDLE_TIMEOUT` equals 2500 ms and `size::SHUTTER` equals 72.0 (guards against token drift).
- [ ] Dependencies added to `Cargo.toml` are only those this ticket needs (GPUI and what it requires). Pin GPUI to an exact version and state the version in the PR description.
- [ ] The app does not enforce floating; window placement is left to the compositor.

**Out of this ticket:** GStreamer, camera access, any widget.
