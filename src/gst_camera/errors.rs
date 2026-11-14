use crate::app_core::backend::CameraError;
use gstreamer as gst;

/// Map a GStreamer error message to the camera error it means.
pub(super) fn camera_error(message: &gst::Message) -> CameraError {
    let gst::MessageView::Error(error) = message.view() else {
        return CameraError::NoCamera;
    };
    match error.error().kind::<gst::ResourceError>() {
        Some(gst::ResourceError::Busy) => CameraError::InUse,
        Some(gst::ResourceError::NotAuthorized) => CameraError::AccessDenied,
        Some(gst::ResourceError::NotFound) => CameraError::NoCamera,
        _ => CameraError::NoCamera,
    }
}
