## Agent skills

### Issue tracker

Issues and specs are local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Project rules

- **Commits:** always via `/commit:commit` (commit only, commit + push, or new branch; a PR requires push).
- **Worktrees:** off by default; only for genuinely parallel or logically required tasks.
- **`/mattpocock-skills:implement <ticket>`:** branch from updated `main` as `<ticket-number>-<kebab-slug>` (e.g. `12-take-photo`), implement, open a PR to `main` unless told "do not open a PR".
- **Scope:** smallest change that fully solves the task, like a lazy senior dev. Rust: small single-purpose fns, intention-revealing names, idiomatic types (`Option`/`Result` over sentinels, enums over flags, `?` over manual matching), no `unwrap`/`expect` in non-test code unless the invariant is documented, borrow before cloning, no duplication or dead code. `cargo clippy` and `cargo fmt` clean.
- **Structure (keep files small):** ~300 lines per file excluding tests, ~40 lines per fn; past that, split. Visual pieces that need no `&mut self` are pure fns in their own module (`mode_pill.rs`, `photo_ui.rs`), not in `camera_window.rs`. Decision logic lives in `app_core` (testable without UI); the view only glues `state.handle(...)` and `cx.notify()`. Fields that travel together become a struct with their methods. A ticket that adds a visual piece creates its module from the start, leaving the root view one `.child(...)` line.
- **Docs:** `///` comment above every fn/method saying what it does (e.g. `/// Greet the user`); comment non-obvious code to ease review.
- **Comments:** only those that add value; no noise.
- **Ticket completion (strict):** when finishing a ticket, check off in its markdown file under `.scratch/<feature>/` only the checkboxes implemented correctly and verified, and append a final `## Implementation notes` section with a comment for each task not completed successfully or not at 100%: what was attempted, what happened (exact error or blocker), whether it is blocking or non-blocking, its urgency to fix (low, medium, high), what remains, and references (file paths with line numbers, commands, commit hashes). Do not add entries for fully completed tasks.
- **Cargo (machine is resource-constrained):** iterate with `cargo check`/`cargo clippy`; run only the relevant tests (`cargo test <name>`) and the full `cargo test` once before finishing. Never `--release`, never run two cargo commands at once, never `cargo clean`.
