//! Orderly recording shutdown when the window or camera goes away.

use super::backend::{CameraBackend, CameraError, VideoSaving};
use super::state::{AppState, Event};

impl AppState {
    /// Lock Capture input and finalize any active Video before permitting exit.
    pub fn request_close(&mut self, camera: &mut impl CameraBackend) -> Option<VideoSaving> {
        self.closing = true;
        self.cancel_countdown();
        self.stop_recording(camera)
    }

    /// Whether closing has been requested and no Video still needs finalization.
    pub fn can_exit(&self) -> bool {
        self.closing && self.video.is_none()
    }

    /// Finalize an interrupted Video before replacing the Viewfinder with its error screen.
    pub fn camera_lost(
        &mut self,
        camera: &mut impl CameraBackend,
        error: CameraError,
    ) -> Option<VideoSaving> {
        self.cancel_countdown();
        self.timer_menu_open = false;
        self.resolution_menu_open = false;
        if self.video.is_some() {
            self.pending_camera_error = Some(error);
            self.stop_recording(camera)
        } else {
            self.handle(Event::CameraFailed(error));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CameraError;
    use crate::app_core::{backend::CameraBackend, fake::FakeCamera, state::*};
    use std::path::Path;

    /// Closing waits for EOS and prevents another Capture even after saving finishes.
    #[test]
    fn close_waits_for_eos() {
        let mut camera = FakeCamera::ready("Camera", vec![]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state.start_recording(&mut camera, Path::new("/tmp"));
        let mut saving = state.request_close(&mut camera).unwrap();
        assert!(!state.can_exit());
        assert_eq!(state.shutter_action(), None);
        assert_eq!(saving.try_recv(), Ok(None));
        camera.complete_eos();
        state.handle(saving.try_recv().unwrap().unwrap().into());
        assert!(state.can_exit());
        assert_eq!(state.shutter_action(), None);
    }
    /// A failed close finalization exits without retrying or presenting a dialog.
    #[test]
    fn close_failed_eos_still_exits() {
        let mut camera = FakeCamera::ready("Camera", vec![]);
        let mut state = AppState::new();
        state.start_recording(&mut camera, Path::new("/tmp"));
        let mut saving = state.request_close(&mut camera).unwrap();
        assert!(state.request_close(&mut camera).is_none());
        camera.fail_eos();
        state.handle(saving.try_recv().unwrap().unwrap().into());
        assert!(state.can_exit());
        assert_eq!(camera.videos_stopped, 1);
    }

    /// An interrupted Video shows feedback and its camera error only after EOS completes.
    #[test]
    fn disconnect_waits_for_eos_success_or_failure() {
        for (success, error) in [
            (true, CameraError::NoCamera),
            (false, CameraError::NoCamera),
            (true, CameraError::Other),
            (false, CameraError::Other),
        ] {
            let mut camera = FakeCamera::ready("Camera", vec![]);
            let mut state = AppState::new();
            state.handle(camera.detect_cameras().into());
            state.start_recording(&mut camera, Path::new("/tmp"));
            let mut saving = state.camera_lost(&mut camera, error).unwrap();
            assert!(matches!(state.camera, CameraState::Ready(_)));
            assert_eq!(state.last_capture, None);
            assert_eq!(state.shutter_action(), None);
            assert_eq!(saving.try_recv(), Ok(None));
            if success {
                camera.complete_eos();
            } else {
                camera.fail_eos();
            }
            state.handle(saving.try_recv().unwrap().unwrap().into());
            assert_eq!(state.camera, CameraState::Error(error));
            assert_eq!(
                matches!(state.last_capture, Some(CaptureOutcome::VideoSaved(_))),
                success
            );
            assert_eq!(state.video, None);
        }
    }

    /// An idle camera loss switches screens immediately and cancels a pending Capture.
    #[test]
    fn disconnect_while_idle() {
        let mut camera = FakeCamera::ready("Camera", vec![]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state.handle(Event::SetTimer(Timer::ThreeSeconds));
        state.press_shutter(std::time::Duration::ZERO);
        assert!(
            state
                .camera_lost(&mut camera, CameraError::AccessDenied)
                .is_none()
        );
        assert_eq!(state.camera, CameraState::Error(CameraError::AccessDenied));
        assert!(state.countdown.is_none());
        assert_eq!(camera.videos_stopped, 0);
    }
}
