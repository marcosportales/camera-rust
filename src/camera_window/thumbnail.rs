//! Connect the Last-Capture Thumbnail to saved Captures and the system opener.

use gpui::{Context, IntoElement, Window};

use super::CameraWindow;

impl CameraWindow {
    /// Draw the Thumbnail with saved feedback and open its folder on activation.
    pub(super) fn thumbnail(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let just_saved = self.feedback.toast.is_some() && self.state.thumbnail_just_saved();
        let image = crate::thumbnail_ui::image(self.state.thumbnail_capture.as_deref(), window, cx);
        crate::thumbnail_ui::control(
            self.state.thumbnail_capture.as_deref(),
            just_saved,
            image,
            cx.listener(|this, _, _, _| {
                crate::capture_store::open_thumbnail_folder(
                    this.state.thumbnail_capture.as_deref(),
                );
            }),
        )
    }
}
