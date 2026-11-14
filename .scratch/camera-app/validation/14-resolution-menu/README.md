# Ticket 14 validation

Validated with the real Integrated_Webcam_HD camera at a 960 × 540 window size.
Its reported MJPG choices on this machine were 1280 × 720, 960 × 540 and 848 × 480;
no 1080p or unreported sizes appeared.

- Opened the menu at 720p: `menu-720p.png`.
- Selected 540p: the menu closed, the control updated, and the Viewfinder resumed: `selected-540p.png`.
- Reopened the menu: the check moved to 540p: `menu-540p.png`.
- Escape and an outside click each dismissed without changing 540p: `dismissed-540p.png`.

Images show only the controls and menu.

Automated checks:

- `cargo test resolution`: 14 passed, including reference-camera ordering and labels,
  successful selection, failed restart rollback, failed rollback error, and real
  GStreamer pipeline release/new frame and Photo dimensions.
- `cargo test`: 79 passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.

Standards review found a missing test doc comment and an oversized settings function;
both were corrected. Spec review found no requirement failures. The control keeps the
explicitly requested chevron-down icon, including while open.
