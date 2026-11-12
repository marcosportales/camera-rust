mod assets;
// Verbatim copy of the design tokens page; later tickets consume the constants.
#[allow(dead_code)]
#[rustfmt::skip]
mod theme;

use gpui::{
    App, Application, Bounds, Context, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgba, size,
};

use assets::Assets;
use theme::{color, font};

/// Default window size in logical px.
const WINDOW_WIDTH: f32 = 960.0;
const WINDOW_HEIGHT: f32 = 540.0;

/// Root view: an empty Camera window filled with the app background.
struct CameraWindow;

impl Render for CameraWindow {
    /// Draw the app background; the Viewfinder and controls arrive in later tickets.
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgba(color::APP_BG))
            .font_family(font::FAMILY)
    }
}

/// Open the Camera window and quit when it closes.
fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        Assets::register_fonts(cx);

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
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("Camera")),
                ..Default::default()
            }),
            ..Default::default()
        };
        // Opening can fail at runtime (e.g. no Vulkan), so report it and exit instead of panicking.
        if let Err(error) = cx.open_window(options, |_, cx| cx.new(|_| CameraWindow)) {
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

    /// Guards against design token drift.
    #[test]
    fn tokens_match_the_design() {
        assert_eq!(motion::IDLE_TIMEOUT, Duration::from_millis(2500));
        assert_eq!(size::SHUTTER, 72.0);
    }
}
