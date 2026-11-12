use std::path::{Path, PathBuf};

use super::backend::{Camera, CameraBackend, CameraError, PhotoError};
use super::resolution::{Resolution, default_resolution};

/// Whether the next Shutter press produces a Photo or a Video.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Photo,
    Video,
}

/// Countdown between pressing the Shutter and the Capture starting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    Off,
    ThreeSeconds,
    TenSeconds,
}

impl Timer {
    /// The next setting in the cycle Off → 3s → 10s → Off.
    fn next(self) -> Self {
        match self {
            Self::Off => Self::ThreeSeconds,
            Self::ThreeSeconds => Self::TenSeconds,
            Self::TenSeconds => Self::Off,
        }
    }
}

/// Where camera detection stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraState {
    Detecting,
    Ready(Camera),
    Error(CameraError),
}

/// Something that happened that the core reacts to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    CameraReady(Camera),
    CameraFailed(CameraError),
    SetMode(Mode),
    CycleTimer,
    ToggleMirror,
    PhotoSaved(PathBuf),
    PhotoFailed,
}

/// What became of the last Photo the user took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhotoOutcome {
    Saved(PathBuf),
    Failed,
}

impl From<Result<Camera, CameraError>> for Event {
    /// Turn a backend detection result into the matching event.
    fn from(detection: Result<Camera, CameraError>) -> Self {
        match detection {
            Ok(camera) => Self::CameraReady(camera),
            Err(error) => Self::CameraFailed(error),
        }
    }
}

impl From<Result<PathBuf, PhotoError>> for Event {
    /// Turn a backend Photo result into the matching event.
    fn from(photo: Result<PathBuf, PhotoError>) -> Self {
        match photo {
            Ok(path) => Self::PhotoSaved(path),
            Err(PhotoError) => Self::PhotoFailed,
        }
    }
}

/// Everything the UI shows, changed only through [`AppState::handle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub mode: Mode,
    pub timer: Timer,
    pub mirror: bool,
    pub resolution: Option<Resolution>,
    pub camera: CameraState,
    pub photo: Option<PhotoOutcome>,
}

impl AppState {
    /// State at app startup: Photo, Timer off, Mirror on, camera detecting.
    pub fn new() -> Self {
        Self {
            mode: Mode::Photo,
            timer: Timer::Off,
            mirror: true,
            resolution: None,
            camera: CameraState::Detecting,
            photo: None,
        }
    }

    /// Take a Photo into `folder` and record whether it was saved.
    pub fn take_photo(&mut self, camera: &mut impl CameraBackend, folder: &Path) {
        self.handle(camera.take_photo(folder).into());
    }

    /// Apply an event to the state.
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::CameraReady(camera) => {
                self.resolution = default_resolution(&camera.resolutions);
                self.camera = CameraState::Ready(camera);
            }
            Event::CameraFailed(error) => self.camera = CameraState::Error(error),
            Event::SetMode(mode) => self.mode = mode,
            Event::CycleTimer => self.timer = self.timer.next(),
            Event::ToggleMirror => self.mirror = !self.mirror,
            Event::PhotoSaved(path) => self.photo = Some(PhotoOutcome::Saved(path)),
            Event::PhotoFailed => self.photo = Some(PhotoOutcome::Failed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_core::backend::CameraBackend;
    use crate::app_core::fake::FakeCamera;

    /// Shorthand for building a Resolution in tests.
    fn res(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }

    /// Run detection on a scripted fake and feed the result to a fresh state.
    fn state_after_detecting(mut camera: FakeCamera) -> AppState {
        let mut state = AppState::new();
        state.handle(camera.detect_cameras().into());
        state
    }

    #[test]
    fn startup_state() {
        let state = AppState::new();
        assert_eq!(state.mode, Mode::Photo);
        assert_eq!(state.timer, Timer::Off);
        assert!(state.mirror);
        assert_eq!(state.resolution, None);
        assert_eq!(state.camera, CameraState::Detecting);
        assert_eq!(state.photo, None);
    }

    #[test]
    fn each_error_kind_moves_to_its_own_error_state() {
        let kinds = [
            CameraError::InUse,
            CameraError::AccessDenied,
            CameraError::NoCamera,
        ];
        let states: Vec<_> = kinds
            .iter()
            .map(|&kind| state_after_detecting(FakeCamera::failing(kind)).camera)
            .collect();
        assert_eq!(
            states,
            [
                CameraState::Error(CameraError::InUse),
                CameraState::Error(CameraError::AccessDenied),
                CameraState::Error(CameraError::NoCamera),
            ]
        );
    }

    #[test]
    fn ready_camera_moves_to_ready_with_the_default_resolution() {
        let resolutions = vec![res(640, 480), res(1280, 720), res(1920, 1080)];
        let state =
            state_after_detecting(FakeCamera::ready("Integrated Camera", resolutions.clone()));
        assert_eq!(
            state.camera,
            CameraState::Ready(Camera {
                name: "Integrated Camera".into(),
                resolutions
            })
        );
        assert_eq!(state.resolution, Some(res(1920, 1080)));
    }

    #[test]
    fn set_mode_switches_between_photo_and_video() {
        let mut state = AppState::new();
        state.handle(Event::SetMode(Mode::Video));
        assert_eq!(state.mode, Mode::Video);
        state.handle(Event::SetMode(Mode::Photo));
        assert_eq!(state.mode, Mode::Photo);
    }

    #[test]
    fn selecting_the_active_mode_changes_nothing() {
        let mut state = AppState::new();
        let before = state.clone();
        state.handle(Event::SetMode(Mode::Photo));
        assert_eq!(state, before);
        state.handle(Event::SetMode(Mode::Video));
        let before = state.clone();
        state.handle(Event::SetMode(Mode::Video));
        assert_eq!(state, before);
    }

    #[test]
    fn timer_cycles_off_3s_10s_and_wraps_to_off() {
        let mut state = AppState::new();
        let mut seen = Vec::new();
        for _ in 0..3 {
            state.handle(Event::CycleTimer);
            seen.push(state.timer);
        }
        assert_eq!(seen, [Timer::ThreeSeconds, Timer::TenSeconds, Timer::Off]);
    }

    #[test]
    fn toggle_mirror_flips_it_each_time() {
        let mut state = AppState::new();
        state.handle(Event::ToggleMirror);
        assert!(!state.mirror);
        state.handle(Event::ToggleMirror);
        assert!(state.mirror);
    }

    #[test]
    fn a_saved_photo_is_recorded_with_its_path() {
        let path = PathBuf::from("/pics/Camera/a.jpg");
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]).saving_photo_as(&path);
        let mut state = AppState::new();
        state.take_photo(&mut camera, Path::new("/pics/Camera"));
        assert_eq!(state.photo, Some(PhotoOutcome::Saved(path)));
        assert_eq!(camera.photo_folders, [PathBuf::from("/pics/Camera")]);
    }

    #[test]
    fn a_failed_photo_is_recorded_as_failed() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]).failing_photo();
        let mut state = AppState::new();
        state.take_photo(&mut camera, Path::new("/pics/Camera"));
        assert_eq!(state.photo, Some(PhotoOutcome::Failed));
    }

    #[test]
    fn every_press_takes_a_photo() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = AppState::new();
        for _ in 0..3 {
            state.take_photo(&mut camera, Path::new("/pics/Camera"));
        }
        assert_eq!(camera.photo_folders.len(), 3);
    }
}
