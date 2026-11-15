use crate::app_core::backend::CameraError;
use gstreamer as gst;

/// Map a GStreamer error message to the camera error it means.
pub(super) fn camera_error(message: &gst::Message) -> CameraError {
    let gst::MessageView::Error(error) = message.view() else {
        return CameraError::Other;
    };
    match error.error().kind::<gst::ResourceError>() {
        Some(gst::ResourceError::Busy) => CameraError::InUse,
        Some(gst::ResourceError::NotAuthorized) => CameraError::AccessDenied,
        Some(gst::ResourceError::NotFound) => CameraError::NoCamera,
        _ => CameraError::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Preserve known resource failures and classify unrelated capture failures separately.
    #[test]
    fn classifies_capture_failures() {
        gst::init().unwrap();
        for (kind, expected) in [
            (gst::ResourceError::Busy, CameraError::InUse),
            (gst::ResourceError::NotAuthorized, CameraError::AccessDenied),
            (gst::ResourceError::NotFound, CameraError::NoCamera),
            (gst::ResourceError::Failed, CameraError::Other),
        ] {
            let message = gst::message::Error::new(kind, "capture failed");
            assert_eq!(camera_error(&message), expected);
        }
        let message = gst::message::Error::new(gst::CoreError::MissingPlugin, "missing plugin");
        assert_eq!(camera_error(&message), CameraError::Other);
    }
}
