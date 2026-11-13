# 07: Video save confirmed by EOS

**What to build:** After stopping a Video, the app shows "Saving…" until the file is finalized, announces success only then, and reports write failures.

**Blocked by:** 06

**Status:** ready-for-agent

**Design reference:** toast and Shutter styles from `html/05-photo-flash.html` and `html/08-components.html`. There is no design screen for "Saving…"; the rules below define it.

- [x] After the user stops a recording, the Shutter is disabled and a text label "Saving…" (`type.small`, `TEXT_SECONDARY`) is shown directly above the Shutter, centered, until the pipeline reports EOS complete.
- [x] While "Saving…" is shown, Space and Shutter clicks do nothing, and no other Capture can start.
- [x] Only after EOS completes: "Saving…" disappears and the toast from ticket 04 appears with text "Video saved" and the folder relative to home (e.g. `Videos/Camera`).
- [x] If finalization or writing fails (disk full, permission, pipeline error): no success toast; the failure toast shape from ticket 04 appears with the text "Couldn't save video"; the Shutter returns to its Video-Mode look.
- [x] The app never shows "Video saved" before EOS completes (core test with a fake backend that delays EOS).
- [x] Core tests: stop → saving → saved; stop → saving → failed; input ignored while saving.

**Out of this ticket:** Thumbnail refresh (ticket 18), close/disconnect during recording (20).

## Implementation notes
