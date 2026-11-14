# 21: Save Captures with Mirror (special spec override)

**Status:** ready-for-agent

**What to build:** Save Photos and Videos with the same horizontal orientation as the Viewfinder: mirrored when Mirror is on, original when it is off.

## User decision — 2026-11-14

The user explicitly overrode the original spec: “no me gustan las fotos sin efecto Mirror, quedo horrible”. This is the user's requested product decision, not an implementation deviation. This ticket takes precedence over spec user story 24 and the Photo architecture rule, ticket 04's no-Mirror/no-re-encoding rule, and ticket 13's view-only/identical-JPEG criteria, for both Photos and Videos. In a follow-up the user explicitly requested “Aplicarlo también a vídeos”.

Mirrored Photos may be decoded and re-encoded as JPEG to bake in the horizontal transformation. With Mirror off, retain the original camera JPEG bytes. Startup defaults, controls, shortcuts and locking remain as specified.

## Acceptance criteria

- [ ] The core passes the current Mirror setting to the backend for immediate and Timer-triggered Photos and Videos. Video orientation is fixed when recording starts.
- [ ] With Mirror on, the saved JPEG has horizontally reversed columns, preserving dimensions and row order.
- [ ] With Mirror off, the saved JPEG matches the camera's source bytes; toggling on/off/on does not mutate that source.
- [ ] Saved Videos follow Mirror on/off at recording start, preserving dimensions and audio/finalization behavior.
- [ ] Encoding/decoding and write failures follow the existing Photo failure feedback without leaving a successful or partial Capture.
- [ ] The spec, glossary and historical tickets link this explicit user override, and replace the original never-mirrored Video requirement.
- [ ] Relevant tests, full `cargo test`, `cargo check`, `cargo clippy` and formatting checks pass; standards and spec reviews are complete.

**Out of this ticket:** Changing the Mirror control, persisting settings.
