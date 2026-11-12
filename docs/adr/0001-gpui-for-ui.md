# Use GPUI for the UI

The app is a desktop Viewfinder for a tiling Wayland compositor, and the design (flat colors, rounded rectangles, SVG icons, fades only, no backdrop blur) was made to fit what GPUI renders well. GPUI is pre-1.0 (crates.io 0.2.x, with Zed's git `main` as the alternative) and requires Vulkan; we accept that lock-in over GTK/egui because the design is already built around it.
