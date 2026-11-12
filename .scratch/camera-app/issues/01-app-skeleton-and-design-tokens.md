# 01: App skeleton and design tokens

**What to build:** A GPUI app that opens an empty Camera window with the project's design tokens, the Inter font and the SVG icon set available to every later ticket. No Viewfinder, no controls.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Design reference:** `docs/design/Camera-UI/html/09-design-tokens.html` (the `theme.rs` block), `docs/design/Camera-UI/html/fonts/`.

**GPUI references:** SVG assets: [`svg/svg.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/svg/svg.rs).

- [x] `cargo run` opens one window titled `Camera`, default size 960 × 540 logical px, resizable, with no custom title bar drawn by the app.
- [x] The window background is `color::APP_BG`.
- [x] Closing the window ends the process with exit code 0.
- [x] A theme module contains the `theme.rs` block from the design tokens page **verbatim**: `color`, `radius`, `space`, `size`, `font`, `motion` modules with every constant and value listed there. Nothing added, renamed or removed.
- [x] Inter weights 300, 400, 500 and 600 from the design's `fonts/` folder are bundled and registered, and `font::FAMILY` resolves to them. No other font is bundled.
- [ ] The Lucide icons `timer`, `flip-horizontal`, `camera`, `camera-off`, `check`, `x`, `rotate-cw`, `play`, `folder` and `chevron-down` are bundled as SVG assets and loadable by name through GPUI's asset source. Stroke width and sizes are applied by the code that draws them, not baked into the files.
- [x] A unit test asserts every icon name above resolves to non-empty SVG data.
- [x] A unit test asserts `motion::IDLE_TIMEOUT` equals 2500 ms and `size::SHUTTER` equals 72.0 (guards against token drift).
- [x] Dependencies added to `Cargo.toml` are only those this ticket needs (GPUI and what it requires). Pin GPUI to an exact version and state the version in the PR description.
- [x] The app does not enforce floating; window placement is left to the compositor.

**Out of this ticket:** GStreamer, camera access, any widget.

## Implementation notes

### The Lucide icons ... Stroke width and sizes are applied by the code that draws them, not baked into the files (unchecked)

- **Attempted:** bundled the 10 stock Lucide SVGs (`lucide-static@0.468.0`) in `assets/icons/`, served by `Assets::load` (`src/assets.rs`); the test `every_required_icon_loads_as_non_empty_svg` passes.
- **What happened:** the files keep `stroke-width="2"` and `width="24" height="24"` (e.g. `assets/icons/timer.svg:8-10`). GPUI's `svg()` only recolours and resizes, so stroke width cannot be set at draw time, and the design also uses 1.75.
- **Remains:** decide how later tickets get the 1.75 stroke: ship per-icon modified files, or strip the attributes and accept 2 everywhere. Ticket wording cannot be met as written with GPUI 0.2.2.
- **References:** `assets/icons/timer.svg:10`; `~/.cargo/registry/src/*/gpui-0.2.2/src/elements/svg.rs`; PR #1 notes.

