Design the UI for a desktop webcam app called "Camera", for Arch Linux running Hyprland (a tiling Wayland compositor). It will be implemented in Rust with the GPUI library (from the Zed editor), so the design must respect what GPUI renders well:
- Flat or semi-transparent solid colors, rounded rectangles, simple shadows, simple linear gradients, text, and SVG icons.
- NO backdrop blur, no CSS-only tricks, no complex animations (only fades and color changes).

## Product
A viewfinder for taking photos and recording videos with the computer's single built-in webcam. Think of a phone camera app adapted to the desktop. MVP only: no filters, zoom, manual exposure, camera selector or built-in gallery.

## Window
- Default 16:9 (about 960x540), resizable and floating. No custom title bar (the compositor manages the window).
- The live camera image (Viewfinder) fills the whole window, with black letterbox bars if the aspect ratio doesn't match.
- Dark theme only. One accent color: red (recording). Everything else is white/gray over the image.

## Controls (overlaid on the Viewfinder, they fade out after a few seconds without mouse movement)
- Bottom row, centered: Last-Capture Thumbnail on the left, big round Shutter in the center, Mirror toggle on the right.
- A pill with two separate option buttons, "Photo | Video", just above the Shutter (the active Mode is highlighted).
- Top right: small icon buttons for Timer (off / 3s / 10s) and Resolution; each opens a small menu.
- Last-Capture Thumbnail: small rounded image of the latest capture; for a video it shows a frame with a small play icon. Clicking it opens the capture folder in the file manager (no in-app gallery).
- Keyboard-first: Space = Shutter, P / V = Photo / Video mode, T = cycle Timer, M = Mirror, Esc = cancel countdown. Show these as tooltips on hover (no separate help screen).
- Show the active camera name discreetly (e.g. small text, top left).

## States to design (one screen each)
1. Ready, Photo mode
2. Ready, Video mode
3. Timer countdown (big number over the Viewfinder)
4. Recording video (Shutter turns red/square, red dot + elapsed time)
5. Brief flash/confirmation after taking a photo
6. Camera unavailable / busy / permission denied (error screen with a Retry button)
7. Timer menu and Resolution menu open

## Deliverables
- The 7 screens above, plus a dark-mode component sheet (Shutter, mode pill, icon buttons, menus, thumbnail, tooltip).
- An explicit design-token list at the end, easy to port to code: colors (with opacity values), corner radii, spacing scale, font sizes/weights, icon sizes, and fade durations.
- Keep the visual language minimal, modern and calm, so we can iterate on it afterwards.
