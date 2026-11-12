use super::backend::{Camera, CameraError};
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

/// Everything the UI shows, changed only through [`AppState::handle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub mode: Mode,
    pub timer: Timer,
    pub mirror: bool,
    pub resolution: Option<Resolution>,
    pub camera: CameraState,
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
        }
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
}
