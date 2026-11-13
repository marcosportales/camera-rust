use std::path::{Path, PathBuf};

use super::backend::{Camera, CameraBackend, CameraError, PhotoError};
pub use super::recording::Recording;
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

/// What pressing the Shutter does now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutterAction {
    TakePhoto,
    StartRecording,
    StopRecording,
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
    RecordingStarted,
    /// One more second of the Video was recorded.
    RecordingTick,
    RecordingStopped,
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
    pub recording: Option<Recording>,
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
            recording: None,
        }
    }

    /// Take a Photo into `folder` and record whether it was saved.
    pub fn take_photo(&mut self, camera: &mut impl CameraBackend, folder: &Path) {
        self.handle(camera.take_photo(folder).into());
    }

    /// What the Shutter does: nothing until the camera is ready, then stop a recording in progress,
    /// else act on the Mode.
    pub fn shutter_action(&self) -> Option<ShutterAction> {
        if !matches!(self.camera, CameraState::Ready(_)) {
            return None;
        }
        Some(match (self.recording, self.mode) {
            (Some(_), _) => ShutterAction::StopRecording,
            (None, Mode::Photo) => ShutterAction::TakePhoto,
            (None, Mode::Video) => ShutterAction::StartRecording,
        })
    }

    /// Start recording a Video into `folder`; a failure to start leaves the state idle.
    pub fn start_recording(&mut self, camera: &mut impl CameraBackend, folder: &Path) {
        // The write-failure notice is ticket 07; for now a failed start just stays idle.
        if camera.start_video(folder).is_ok() {
            self.handle(Event::RecordingStarted);
        }
    }

    /// Stop the Video being recorded; does nothing when not recording.
    pub fn stop_recording(&mut self, camera: &mut impl CameraBackend) {
        if self.recording.is_some() {
            camera.stop_video();
            self.handle(Event::RecordingStopped);
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
            // The Mode stays put while a Video records, so its Shutter look returns after the stop.
            Event::SetMode(mode) if self.recording.is_none() => self.mode = mode,
            Event::SetMode(_) => {}
            Event::CycleTimer => self.timer = self.timer.next(),
            Event::ToggleMirror => self.mirror = !self.mirror,
            Event::PhotoSaved(path) => self.photo = Some(PhotoOutcome::Saved(path)),
            Event::PhotoFailed => self.photo = Some(PhotoOutcome::Failed),
            Event::RecordingStarted => self.recording = Some(Recording { elapsed_secs: 0 }),
            Event::RecordingTick => {
                if let Some(recording) = &mut self.recording {
                    recording.elapsed_secs += 1;
                }
            }
            Event::RecordingStopped => self.recording = None,
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

    /// A state with a ready camera in Video Mode.
    fn video_state() -> AppState {
        let mut state = state_after_detecting(FakeCamera::ready("Cam", vec![res(1280, 720)]));
        state.handle(Event::SetMode(Mode::Video));
        state
    }

    #[test]
    fn the_shutter_takes_a_photo_in_photo_mode_and_starts_recording_in_video_mode() {
        let mut state = video_state();
        assert_eq!(state.shutter_action(), Some(ShutterAction::StartRecording));
        state.handle(Event::SetMode(Mode::Photo));
        assert_eq!(state.shutter_action(), Some(ShutterAction::TakePhoto));
    }

    #[test]
    fn the_shutter_does_nothing_until_the_camera_is_ready() {
        assert_eq!(AppState::new().shutter_action(), None);
    }

    #[test]
    fn starting_a_recording_asks_the_backend_for_a_video_in_the_folder() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        assert_eq!(state.recording, Some(Recording { elapsed_secs: 0 }));
        assert_eq!(camera.video_folders, [PathBuf::from("/videos/Camera")]);
        assert_eq!(state.shutter_action(), Some(ShutterAction::StopRecording));
    }

    #[test]
    fn a_recording_that_fails_to_start_leaves_the_state_idle() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]).failing_video();
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        assert_eq!(state.recording, None);
    }

    #[test]
    fn stopping_tells_the_backend_and_returns_to_the_video_shutter() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        state.stop_recording(&mut camera);
        assert_eq!(state.recording, None);
        assert_eq!(camera.videos_stopped, 1);
        assert_eq!(state.shutter_action(), Some(ShutterAction::StartRecording));
    }

    #[test]
    fn elapsed_seconds_tick_while_recording() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        for _ in 0..3 {
            state.handle(Event::RecordingTick);
        }
        assert_eq!(state.recording, Some(Recording { elapsed_secs: 3 }));
    }

    #[test]
    fn stop_and_tick_are_ignored_when_not_recording() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        let before = state.clone();
        state.stop_recording(&mut camera);
        state.handle(Event::RecordingStopped);
        state.handle(Event::RecordingTick);
        assert_eq!(state, before);
        assert_eq!(camera.videos_stopped, 0);
    }

    #[test]
    fn the_mode_cannot_change_while_recording() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        state.handle(Event::SetMode(Mode::Photo));
        assert_eq!(state.mode, Mode::Video);
    }

    #[test]
    fn timecode_is_minutes_and_seconds_and_minutes_keep_counting_past_59() {
        let timecode = |elapsed_secs| Recording { elapsed_secs }.timecode();
        assert_eq!(timecode(0), "00:00");
        assert_eq!(timecode(42), "00:42");
        assert_eq!(timecode(3665), "61:05");
    }
}
