#[allow(dead_code)]
mod app_core;
mod assets;
mod camera_window;
mod capture_store;
mod controls_ui;
mod countdown_ui;
mod feedback;
mod gst_camera;
mod gst_recording;
mod locked_controls;
mod menu_ui;
mod mirror_ui;
mod mode_pill;
mod overlays;
mod photo_jpeg;
mod photo_ui;
mod recording_ui;
mod resolution_ui;
mod thumbnail_ui;
mod timer_ui;
mod tooltip_ui;
mod viewfinder;
// Verbatim copy of the design tokens page; later tickets consume the constants.
#[allow(dead_code)]
#[rustfmt::skip]
mod theme;

use gpui::{
    App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, prelude::*, px, size,
};

use assets::Assets;
use camera_window::{
    CameraWindow, CloseTimerMenu, CycleTimer, KEY_CONTEXT, PressShutter, SelectPhoto, SelectVideo,
    ToggleMirror,
};

/// Default window size in logical px.
const WINDOW_WIDTH: f32 = 960.0;
const WINDOW_HEIGHT: f32 = 540.0;

/// Bind the Camera window's shortcuts to the same actions as its controls.
fn key_bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("space", PressShutter, Some(KEY_CONTEXT)),
        KeyBinding::new("p", SelectPhoto, Some(KEY_CONTEXT)),
        KeyBinding::new("v", SelectVideo, Some(KEY_CONTEXT)),
        KeyBinding::new("m", ToggleMirror, Some(KEY_CONTEXT)),
        KeyBinding::new("t", CycleTimer, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", CloseTimerMenu, Some(KEY_CONTEXT)),
    ]
}

/// Open the Camera window and quit when it closes.
fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        Assets::register_fonts(cx);
        cx.bind_keys(key_bindings());

        // Quit on the last window close so the process exits with code 0.
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        // Opening can fail at runtime (e.g. no Vulkan), so report it and exit instead of panicking.
        let opened = cx.open_window(options, |window, cx| {
            // `WindowOptions::titlebar` is ignored on Wayland in GPUI 0.2.2, so set the title explicitly.
            window.set_window_title("Camera");
            cx.new(|cx| CameraWindow::new(window, cx))
        });
        if let Err(error) = opened {
            eprintln!("Could not open the Camera window: {error:#}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::theme::{motion, size};
    use std::time::Duration;

    /// M dispatches Mirror's action, and the core event toggles it from its startup value.
    #[test]
    fn m_shortcut_toggles_mirror() {
        let binding = super::key_bindings()
            .into_iter()
            .find(|binding| {
                binding.match_keystrokes(&[gpui::Keystroke::parse("m").unwrap()]) == Some(false)
            })
            .unwrap();
        assert!(binding.action().as_any().is::<super::ToggleMirror>());
        let mut state = super::app_core::state::AppState::new();
        assert!(state.mirror);
        state.handle(super::app_core::state::Event::ToggleMirror);
        assert!(!state.mirror);
        state.handle(super::app_core::state::Event::ToggleMirror);
        assert!(state.mirror);
    }

    /// Guards against design token drift.
    #[test]
    fn tokens_match_the_design() {
        assert_eq!(motion::IDLE_TIMEOUT, Duration::from_millis(2500));
        assert_eq!(size::SHUTTER, 72.0);
    }
}
