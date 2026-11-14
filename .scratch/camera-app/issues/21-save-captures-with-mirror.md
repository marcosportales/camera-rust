# 21: Save Captures with Mirror (special spec override)

**Status:** ready-for-agent

**What to build:** Save Photos and Videos with the same horizontal orientation as the Viewfinder: mirrored when Mirror is on, original when it is off.

## User decision — 2026-11-14

The user explicitly overrode the original spec: “no me gustan las fotos sin efecto Mirror, quedo horrible”. This is the user's requested product decision, not an implementation deviation. This ticket takes precedence over spec user story 24 and the Photo architecture rule, ticket 04's no-Mirror/no-re-encoding rule, and ticket 13's view-only/identical-JPEG criteria, for both Photos and Videos. In a follow-up the user explicitly requested “Aplicarlo también a vídeos”.

Mirrored Photos may be decoded and re-encoded as JPEG to bake in the horizontal transformation. With Mirror off, retain the original camera JPEG bytes. Startup defaults, controls, shortcuts and locking remain as specified.

## Acceptance criteria

- [x] The core passes the current Mirror setting to the backend for immediate and Timer-triggered Photos and Videos. Video orientation is fixed when recording starts.
- [x] With Mirror on, the saved JPEG has horizontally reversed columns, preserving dimensions and row order.
- [x] With Mirror off, the saved JPEG matches the camera's source bytes; toggling on/off/on does not mutate that source.
- [x] Saved Videos follow Mirror on/off at recording start, preserving dimensions and audio/finalization behavior.
- [x] Encoding/decoding and write failures follow the existing Photo failure feedback without leaving a successful or partial Capture.
- [x] The spec, glossary and historical tickets link this explicit user override, and replace the original never-mirrored Video requirement.
- [x] Relevant tests, full `cargo test`, `cargo check`, `cargo clippy` and formatting checks pass; standards and spec reviews are complete.

**Out of this ticket:** Changing the Mirror control, persisting settings.

## Verification

- `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `git diff --check` passed.
- `cargo test`: 73 passed. After extracting the shared Timer test helper following review, `cargo test mirror`: 7 passed; Clippy and formatting passed again.
- Both saved-orientation tests failed against the prior behavior before implementation, then passed with actual JPEG/MP4 decoding. Tests cover immediate/Timer captures, Mirror locking during Video, JPEG byte preservation, invalid JPEG and unwritable destination failures. Existing audio/finalization tests passed in the full suite.
- Parallel standards and spec reviews against `main` (`14d632b`) completed. Standards: one minor duplicated-test setup finding, fixed and re-reviewed with no remaining findings. Spec: no findings.

## Implementation notes
