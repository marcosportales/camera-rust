## Agent skills

### Issue tracker

Issues and specs are local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Project rules

- **Commits:** always via `/commit:commit`. Pick the mode that fits: commit only, commit + push, or new branch. Opening a PR requires commit + push.
- **Worktrees:** off by default. Use only for genuinely parallel tasks or when one is logically required.
- **`/mattpocock-skills:implement <ticket>`:** branch fresh from updated `main` as `<ticket-number>-<kebab-slug>` (e.g. `12-take-photo`), implement, then open a PR to `main`. Skip the PR only if told "do not open a PR".
- **Scope:** smallest change that fully solves the task, with quality, like a lazy senior dev. Apply clean code practices adapted to Rust: small single-purpose functions, intention-revealing names, idiomatic types (`Option`/`Result` over sentinels, enums over flags, `?` over manual matching), no `unwrap`/`expect` in non-test code unless the invariant is documented, borrow before cloning, and no duplication or dead code. Keep `cargo clippy` and `cargo fmt` clean.
- **Docs:** put a `///` comment above every fn/method saying what it does (e.g. `/// Greet the user`). Comment non-obvious parts of the code to ease review.
- **No noise comments:** skip unnecessary or sloppy comments; every comment must add value.
