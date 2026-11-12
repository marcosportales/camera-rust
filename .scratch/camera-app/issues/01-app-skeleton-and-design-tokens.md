# 01: App skeleton and design tokens

**What to build:** A GPUI app that opens an empty Camera window with the project's design tokens, the Inter font and the SVG icon set available to every later ticket. No Viewfinder, no controls.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Design reference:** `docs/design/Camera-UI/html/09-design-tokens.html` (the `theme.rs` block), `docs/design/Camera-UI/html/fonts/`.

**GPUI references:** SVG assets: [`svg/svg.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/svg/svg.rs).

- [ ] `cargo run` opens one window titled `Camera`, default size 960 × 540 logical px, resizable, with no custom title bar drawn by the app.
- [ ] The window background is `color::APP_BG`.
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

### `cargo run` opens one window titled `Camera`, 960 × 540, resizable, no custom title bar (unchecked)

- **Attempted:** window opened with `Bounds::centered` of 960 × 540 (`src/main.rs:16-17,45`), title set with `window.set_window_title("Camera")` (`src/main.rs:53`), root view is a plain `div` (no custom title bar). Observed with `hyprctl clients -j`: title `Camera` after the fix.
- **What happened:** the title was empty on first run because GPUI 0.2.2 ignores `WindowOptions::titlebar` on Wayland (only `x11/window.rs` reads it); fixed in `5a8d3d6`. The 960 × 540 size and resizability could not be confirmed: Hyprland here floats every window at 1838 × 1048 through user rules, so the requested size is overridden.
- **Remains:** confirm the initial 960 × 540 size and that the window resizes, on a compositor without a size rule (or with a temporary one), and by eye.
- **References:** `src/main.rs:16-17,45,53`; commit `5a8d3d6`; `hyprctl clients -j` showed `size: [1838, 1048]`.

### The window background is `color::APP_BG` (unchecked)

- **Attempted:** `.bg(rgba(color::APP_BG))` on the root `div` (`src/main.rs:27`); tried to capture the window with `grim`.
- **What happened:** both captures showed another window or workspace (the app window opens on a workspace that was not on screen), so the pixel colour was never read. No exact error; the evidence is missing, not failing.
- **Remains:** capture the Camera window and check that its pixels are `#0B0B0C`.
- **References:** `src/main.rs:27`; `grim` capture attempts (deleted, they showed unrelated desktop content).

### The Lucide icons ... Stroke width and sizes are applied by the code that draws them, not baked into the files (unchecked)

- **Attempted:** bundled the 10 stock Lucide SVGs (`lucide-static@0.468.0`) in `assets/icons/`, served by `Assets::load` (`src/assets.rs`); the test `every_required_icon_loads_as_non_empty_svg` passes.
- **What happened:** the files keep `stroke-width="2"` and `width="24" height="24"` (e.g. `assets/icons/timer.svg:8-10`). GPUI's `svg()` only recolours and resizes, so stroke width cannot be set at draw time, and the design also uses 1.75.
- **Remains:** decide how later tickets get the 1.75 stroke: ship per-icon modified files, or strip the attributes and accept 2 everywhere. Ticket wording cannot be met as written with GPUI 0.2.2.
- **References:** `assets/icons/timer.svg:10`; `~/.cargo/registry/src/*/gpui-0.2.2/src/elements/svg.rs`; PR #1 notes.

