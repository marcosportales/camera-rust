# GPUI agent skills and llms.txt: findings

Status: resolved

Research started on 2026-11-12 during the to-tickets session; continued the same day (see "Second pass"). Decision: **option 1, install nothing.** Links to the matching GPUI docs and examples were added as a `GPUI references` line to the tickets that need them.

## Question

Does GPUI ship skills or an `llms.txt` that teach agents to use it correctly? If so, which are the minimum needed for the `camera-app` spec (plain GPUI, custom design tokens, no component library, ADR 0001)?

## Verified facts

- **No GPUI `llms.txt` exists.** `https://gpui.rs/llms.txt` answers 200 but returns the site's HTML page, not an `llms.txt`. `https://zed.dev/llms.txt` is a real file, but it indexes the Zed editor docs, not the GPUI framework.
- **Zed's repo ships agent skills**, in `.agents/skills/` of `zed-industries/zed`: `gpui-test`, `gpui-bench`, `lint-creator`, `zed-cherry-pick`. Only `gpui-test` and `gpui-bench` concern GPUI. `gpui-test` covers `#[gpui::test]`, `TestAppContext`, scheduler seeds, `SEED`/`ITERATIONS` reproduction and flaky-test debugging.
- **Official GPUI docs inside the repo** (not skills): `crates/gpui/docs/contexts.md` and `crates/gpui/docs/key_dispatch.md` in `zed-industries/zed`.
- **`longbridge/gpui-component` ships two skills today:** `gpui-kit` and `gpui-kit-design-guides`. They target the `gpui-kit` crate, a component framework layered on GPUI, with its own `llms.txt` at `https://gpui-kit.com/llms.txt`. Third-party indexes still list older skills from that repo (`gpui-element`, `gpui-context`, `gpui-global`, `new-component`); they are no longer in the repo and were not verified.
- `gpui` on crates.io is at 0.2.2 (published 2026-10-30), pre-1.0.

## Second pass (2026-11-12)

- **Older `gpui-component` skills confirmed gone.** `skills/` in `longbridge/gpui-component` contains only `gpui-kit` and `gpui-kit-design-guides`. The third-party listings of `gpui-element`, `gpui-context`, `gpui-global`, `new-component` are stale.
- **`gpui-kit/references/`** holds `coding-guides.md`, `conventions.md`, `recipes.md`, `usage.md` and a `gpui/` folder. Not read in depth; still the only candidate for option 3.
- **The topics the tickets need with no skill are covered by runnable examples** in `crates/gpui/examples/` of `zed-industries/zed`:
  - Actions and key bindings: `input.rs` (`actions!`, `cx.bind_keys([KeyBinding::new(...)])`).
  - Opacity animation: `opacity.rs` (`.opacity(..)`), `animation.rs` (`with_animation`, `Animation::new`, spring animations, animated `svg()`).
  - SVG assets: `svg/svg.rs` and `animation.rs`.
  - Drawing images: `image/`, `image_loading.rs` (custom `ImageAssetLoader`, `RenderImage`, `img(...)`), `gif_viewer.rs`. For a live decoded frame, `Window::paint_image` exists in `window.rs`; no example feeds frames from a camera, so the frame-to-`RenderImage` path needs a spike in the ticket that implements the viewfinder.
  - Focus and key context: `focus_visible.rs`, `tab_stop.rs`, plus `docs/key_dispatch.md`.
  - Async: not checked per example. Spawning from entities is covered by `docs/contexts.md`.
- **Gap that nothing covers:** GStreamer-to-GPUI frame handoff. No skill, doc or example found. It belongs to a spike ticket, not to an installed skill.

## Fit with this project

- The spec uses plain GPUI with the tokens in `docs/design/Camera-UI`. Installing `gpui-kit` would steer agents toward its components and theme, against ADR 0001 and the design. The Design Guides skill would add a second source of design truth next to our own.
- The project's test seam is the pure core, not GPUI (`spec.md`, Testing Decisions), so `gpui-test` is only needed if a ticket later tests a GPUI entity.
- What agents need from GPUI for the tickets: actions and key bindings (Space, P, V, T, M, Esc), contexts and entities, async tasks (GStreamer events, timers), drawing a decoded frame, SVG assets, opacity animations. The two in-repo docs cover contexts and key dispatch; the other topics have no official skill.

## Options for the later session

1. **Install nothing.** Link `contexts.md` and `key_dispatch.md` from the tickets that need them and rely on the GPUI examples in the Zed repo. Lowest cost, no conflict with ADR 0001.
2. **Install `gpui-test` only**, if the project decides to test GPUI entities.
3. **Read, not install**, `skills/gpui-kit/references/coding-guides.md` (sections on contexts, state ownership, events/focus, async) as a one-off reference. Some of it is `gpui-kit`-specific.

Recommended: option 1, now with concrete links. In each ticket, point to the matching example file above instead of installing anything. Revisit option 2 when a ticket needs a GPUI-level test.

## Sources

- https://zed.dev/docs/ai/skills
- https://zed.dev/llms.txt
- https://github.com/zed-industries/zed/tree/main/.agents/skills
- https://github.com/zed-industries/zed/tree/main/crates/gpui/docs
- https://github.com/longbridge/gpui-component/tree/main/skills
- https://gpui-kit.com/llms.txt
- https://crates.io/crates/gpui
