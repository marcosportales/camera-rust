use super::backend::CameraBackend;
use super::state::{AppState, CameraState, Event};

impl AppState {
    /// Detect again and start the Viewfinder, retaining the device name if startup fails.
    pub fn retry_camera(&mut self, camera: &mut impl CameraBackend) {
        if self.video.is_some() {
            return;
        }
        self.camera = CameraState::Detecting;
        self.resolution = None;
        self.countdown = None;
        self.timer_menu_open = false;
        self.resolution_menu_open = false;
        self.handle(camera.detect_cameras().into());
        if let Some(resolution) = self.resolution
            && let Err(error) = camera.start_viewfinder(resolution)
        {
            self.handle(Event::CameraFailed(error));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_core::{backend::CameraError, fake::FakeCamera, resolution::Resolution};
    /// Retrying detects and starts again, including after a different startup failure.
    #[test]
    fn retry_recovers_and_reports_new_errors() {
        let mut state = AppState::new();
        let mut camera = FakeCamera::ready(
            "Webcam",
            vec![Resolution {
                width: 640,
                height: 480,
            }],
        );
        camera.start_results.extend([
            Err(CameraError::InUse),
            Err(CameraError::AccessDenied),
            Ok(()),
        ]);
        state.retry_camera(&mut camera);
        assert_eq!(state.camera, CameraState::Error(CameraError::InUse));
        state.retry_camera(&mut camera);
        assert_eq!(state.camera, CameraState::Error(CameraError::AccessDenied));
        state.retry_camera(&mut camera);
        assert!(matches!(state.camera, CameraState::Ready(_)));
        state.retry_camera(&mut FakeCamera::failing(CameraError::NoCamera));
        assert_eq!(state.camera, CameraState::Error(CameraError::NoCamera));
        assert_eq!(state.resolution, None);
    }
}
