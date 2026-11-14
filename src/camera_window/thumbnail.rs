//! Connect the Last-Capture Thumbnail to saved Captures and the system opener.

use gpui::{Context, IntoElement};

use super::CameraWindow;

impl CameraWindow {
    /// Draw the Thumbnail with saved feedback and open its folder on activation.
    pub(super) fn thumbnail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let just_saved = self.feedback.toast.is_some() && self.state.thumbnail_just_saved();
        crate::thumbnail_ui::control(
            self.state.thumbnail_photo.as_deref(),
            just_saved,
            cx.listener(|this, _, _, _| {
                crate::capture_store::open_thumbnail_folder(this.state.thumbnail_photo.as_deref());
            }),
        )
    }
}
