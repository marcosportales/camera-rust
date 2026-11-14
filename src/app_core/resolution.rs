use super::backend::{CameraBackend, CameraError};
use super::state::{AppState, CameraState};

/// Pixel size at which the camera delivers the Viewfinder and Captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    /// Whether this Resolution is 16:9, within a 1% tolerance.
    pub fn is_16_9(self) -> bool {
        // `width * 9` vs `height * 16`, compared in integers: |a - b| <= 1% of b.
        let (a, b) = (u64::from(self.width) * 9, u64::from(self.height) * 16);
        a.abs_diff(b) * 100 <= b
    }

    /// The short label used on the control and in menu rows.
    pub fn label(self) -> String {
        format!("{}p", self.height)
    }

    /// Pixel dimensions, with a reduced aspect ratio for sizes outside the 16:9 tolerance.
    pub fn description(self) -> String {
        let dimensions = format!("{} × {}", self.width, self.height);
        if self.is_16_9() {
            return dimensions;
        }
        let (mut a, mut b) = (self.width, self.height);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let divisor = a.max(1);
        format!(
            "{dimensions} · {}:{}",
            self.width / divisor,
            self.height / divisor
        )
    }

    /// Total pixel count, used to rank Resolutions.
    pub fn pixels(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

impl AppState {
    /// Restart at a supported Resolution, preserving the previous selection on failure.
    pub fn select_resolution(
        &mut self,
        resolution: Resolution,
        camera: &mut impl CameraBackend,
    ) -> Result<(), CameraError> {
        if self.control_lock().is_some() {
            return Ok(());
        }
        self.resolution_menu_open = false;
        let Some(previous) = self.resolution else {
            return Ok(());
        };
        if resolution == previous || !self.resolution_choices().contains(&resolution) {
            return Ok(());
        }
        if let Err(error) = camera.start_viewfinder(resolution) {
            if let Err(restore_error) = camera.start_viewfinder(previous) {
                self.camera = CameraState::Error(restore_error);
            }
            return Err(error);
        }
        self.resolution = Some(resolution);
        Ok(())
    }

    /// Only the detected camera's MJPG Resolutions, sorted by descending pixel count.
    pub fn resolution_choices(&self) -> Vec<Resolution> {
        let CameraState::Ready(camera) = &self.camera else {
            return Vec::new();
        };
        let mut choices = camera.resolutions.clone();
        choices.sort_by_key(|resolution| std::cmp::Reverse(resolution.pixels()));
        choices.dedup();
        choices
    }
}

/// Pick the default Resolution: the biggest 16:9 one, else the biggest overall.
pub fn default_resolution(resolutions: &[Resolution]) -> Option<Resolution> {
    let biggest_16_9 = resolutions
        .iter()
        .copied()
        .filter(|r| r.is_16_9())
        .max_by_key(|r| r.pixels());
    biggest_16_9.or_else(|| resolutions.iter().copied().max_by_key(|r| r.pixels()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand for building a Resolution in tests.
    fn res(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }

    /// Dismissal and unsupported or unchanged choices preserve the Viewfinder without a restart.
    #[test]
    fn resolution_menu_dismisses_without_changing_selection() {
        use crate::app_core::fake::FakeCamera;
        use crate::app_core::state::Event;
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state.handle(Event::ToggleResolutionMenu);
        state.handle(Event::CloseResolutionMenu);
        assert!(!state.resolution_menu_open);
        state.handle(Event::ToggleResolutionMenu);
        state.handle(Event::ToggleTimerMenu);
        assert!(!state.resolution_menu_open);
        state.handle(Event::ToggleResolutionMenu);
        assert!(!state.timer_menu_open);
        for choice in [res(1280, 720), res(1920, 1080)] {
            state.select_resolution(choice, &mut camera).unwrap();
            assert_eq!(state.resolution, Some(res(1280, 720)));
            assert!(!state.resolution_menu_open);
        }
        assert!(camera.started_at.is_empty());
    }

    /// Losing the camera during rollback surfaces its error instead of reporting a working Viewfinder.
    #[test]
    fn resolution_failed_rollback_reports_the_camera_error() {
        use crate::app_core::fake::FakeCamera;
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720), res(640, 480)]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        camera
            .start_results
            .extend([Err(CameraError::InUse), Err(CameraError::AccessDenied)]);
        assert_eq!(
            state.select_resolution(res(640, 480), &mut camera),
            Err(CameraError::InUse)
        );
        assert_eq!(state.resolution, Some(res(1280, 720)));
        assert_eq!(state.camera, CameraState::Error(CameraError::AccessDenied));
    }

    /// A rejected restart restores the previous Viewfinder and preserves its checked selection.
    #[test]
    fn resolution_failure_restarts_the_previous_size() {
        use crate::app_core::fake::FakeCamera;
        use crate::app_core::state::Event;
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720), res(640, 480)]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state.handle(Event::ToggleResolutionMenu);
        camera.start_results.push_back(Err(CameraError::InUse));
        assert_eq!(
            state.select_resolution(res(640, 480), &mut camera),
            Err(CameraError::InUse)
        );
        assert_eq!(state.resolution, Some(res(1280, 720)));
        assert_eq!(camera.started_at, [res(640, 480), res(1280, 720)]);
        assert!(!state.resolution_menu_open);
        assert!(matches!(state.camera, CameraState::Ready(_)));
    }

    /// Selecting a supported size restarts the backend, updates the selection and closes the menu.
    #[test]
    fn resolution_selection_restarts_the_viewfinder() {
        use crate::app_core::backend::CameraBackend;
        use crate::app_core::fake::FakeCamera;
        use crate::app_core::state::{AppState, Event};
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720), res(640, 480)]);
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state.handle(Event::ToggleResolutionMenu);
        assert!(state.resolution_menu_open);
        assert_eq!(state.select_resolution(res(640, 480), &mut camera), Ok(()));
        assert_eq!(state.resolution, Some(res(640, 480)));
        assert_eq!(camera.started_at, [res(640, 480)]);
        assert!(!state.resolution_menu_open);
    }

    /// Reported MJPG sizes are ranked by pixels and labelled using the aspect ratio rule.
    #[test]
    fn resolution_menu_orders_and_labels_the_reference_camera() {
        use crate::app_core::backend::Camera;
        use crate::app_core::state::{AppState, Event};
        let mut state = AppState::new();
        state.handle(Event::CameraReady(Camera {
            name: "Reference camera".into(),
            resolutions: vec![
                res(640, 360),
                res(640, 480),
                res(848, 480),
                res(1280, 720),
                res(960, 540),
            ],
        }));
        let rows: Vec<_> = state
            .resolution_choices()
            .into_iter()
            .map(|resolution| (resolution.label(), resolution.description()))
            .collect();
        assert_eq!(
            rows,
            [
                ("720p".into(), "1280 × 720".into()),
                ("540p".into(), "960 × 540".into()),
                ("480p".into(), "848 × 480".into()),
                ("480p".into(), "640 × 480 · 4:3".into()),
                ("360p".into(), "640 × 360".into()),
            ]
        );
        assert_eq!(res(1280, 1024).description(), "1280 × 1024 · 5:4");
    }

    #[test]
    fn common_16_9_sizes_are_16_9() {
        for (w, h) in [(1280, 720), (960, 540), (848, 480), (640, 360)] {
            assert!(res(w, h).is_16_9(), "{w}x{h}");
        }
    }

    #[test]
    fn four_by_three_is_not_16_9() {
        assert!(!res(640, 480).is_16_9());
    }

    #[test]
    fn default_is_the_biggest_16_9_when_there_are_several() {
        let list = [
            res(640, 360),
            res(1920, 1080),
            res(1280, 720),
            res(2560, 1920),
        ];
        assert_eq!(default_resolution(&list), Some(res(1920, 1080)));
    }

    #[test]
    fn default_is_the_biggest_overall_when_none_is_16_9() {
        let list = [res(640, 480), res(1600, 1200), res(800, 600)];
        assert_eq!(default_resolution(&list), Some(res(1600, 1200)));
    }

    #[test]
    fn default_of_an_empty_list_is_none() {
        assert_eq!(default_resolution(&[]), None);
    }
}
