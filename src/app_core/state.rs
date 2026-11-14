use std::path::{Path, PathBuf};

use super::backend::{Camera, CameraBackend, CameraError, PhotoError, VideoError, VideoSaving};
pub use super::recording::{Recording, VideoCapture};
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
    StartCountdown {
        action: ShutterAction,
        now: std::time::Duration,
        seconds: u64,
    },
    CountdownTick(std::time::Duration),
    CancelCountdown,
    CameraReady(Camera),
    CameraFailed(CameraError),
    SetMode(Mode),
    CycleTimer,
    ToggleTimerMenu,
    CloseTimerMenu,
    SetTimer(Timer),
    ToggleMirror,
    PhotoSaved(PathBuf),
    PhotoFailed,
    RecordingStarted,
    /// One more second of the Video was recorded.
    RecordingTick,
    RecordingStopped,
    /// The stopped Video's file was finalized.
    VideoSaved(PathBuf),
    /// The Video could not be started or finalized.
    VideoFailed,
}

/// What became of the last Capture the user took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureOutcome {
    PhotoSaved(PathBuf),
    PhotoFailed,
    VideoSaved(PathBuf),
    VideoFailed,
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

impl From<Result<PathBuf, VideoError>> for Event {
    /// Turn a Video finalization result into the matching event.
    fn from(video: Result<PathBuf, VideoError>) -> Self {
        match video {
            Ok(path) => Self::VideoSaved(path),
            Err(VideoError) => Self::VideoFailed,
        }
    }
}

/// Everything the UI shows, changed only through [`AppState::handle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub countdown: Option<super::countdown::Countdown>,
    pub mode: Mode,
    pub timer: Timer,
    pub timer_menu_open: bool,
    pub mirror: bool,
    pub resolution: Option<Resolution>,
    pub camera: CameraState,
    pub last_capture: Option<CaptureOutcome>,
    pub video: Option<VideoCapture>,
}

impl AppState {
    /// State at app startup: Photo, Timer off, Mirror on, camera detecting.
    pub fn new() -> Self {
        Self {
            countdown: None,
            mode: Mode::Photo,
            timer: Timer::Off,
            timer_menu_open: false,
            mirror: true,
            resolution: None,
            camera: CameraState::Detecting,
            last_capture: None,
            video: None,
        }
    }

    /// The Video being recorded, if one is (not while it is saving).
    pub fn recording(&self) -> Option<Recording> {
        match self.video {
            Some(VideoCapture::Recording(recording)) => Some(recording),
            _ => None,
        }
    }

    /// Take a Photo into `folder` unless a Video is recording or saving, and record its outcome.
    pub fn take_photo(&mut self, camera: &mut impl CameraBackend, folder: &Path) {
        if self.video.is_some() {
            return;
        }
        self.handle(camera.take_photo(folder).into());
    }

    /// What the Shutter does: nothing until the camera is ready or while a Video saves, then stop a
    /// recording in progress, else act on the Mode.
    pub fn shutter_action(&self) -> Option<ShutterAction> {
        if !matches!(self.camera, CameraState::Ready(_)) {
            return None;
        }
        match (self.video, self.mode) {
            (Some(VideoCapture::Saving), _) => None,
            (Some(VideoCapture::Recording(_)), _) => Some(ShutterAction::StopRecording),
            (None, Mode::Photo) => Some(ShutterAction::TakePhoto),
            (None, Mode::Video) => Some(ShutterAction::StartRecording),
        }
    }

    /// Start a Video into `folder` unless one is recording or saving; report any start failure.
    pub fn start_recording(&mut self, camera: &mut impl CameraBackend, folder: &Path) {
        if self.video.is_some() {
            return;
        }
        let event = match camera.start_video(folder) {
            Ok(_) => Event::RecordingStarted,
            Err(VideoError) => Event::VideoFailed,
        };
        self.handle(event);
    }

    /// Stop the Video being recorded and start saving it; returns what resolves once it is finalized,
    /// to be fed back as an event. Does nothing when not recording.
    pub fn stop_recording(&mut self, camera: &mut impl CameraBackend) -> Option<VideoSaving> {
        self.recording()?;
        let saving = camera.stop_video();
        self.handle(Event::RecordingStopped);
        Some(saving)
    }

    /// Apply an event to the state.
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::StartCountdown {
                now,
                seconds,
                action,
            } => {
                self.timer_menu_open = false;
                self.countdown = Some(super::countdown::Countdown {
                    action,
                    deadline: now + std::time::Duration::from_secs(seconds),
                    remaining: seconds as u8,
                });
            }
            Event::CountdownTick(now) => {
                if let Some(countdown) = &mut self.countdown {
                    countdown.remaining =
                        countdown.deadline.saturating_sub(now).as_secs_f64().ceil() as u8;
                }
            }
            Event::CancelCountdown => self.countdown = None,
            Event::CameraReady(camera) => {
                self.resolution = default_resolution(&camera.resolutions);
                self.camera = CameraState::Ready(camera);
            }
            Event::CameraFailed(error) => self.camera = CameraState::Error(error),
            // The Mode stays put while a Video records or saves, so its Shutter look returns after.
            Event::SetMode(mode) if self.video.is_none() => self.mode = mode,
            Event::SetMode(_) => {}
            Event::CycleTimer => self.timer = self.timer.next(),
            Event::ToggleTimerMenu => self.timer_menu_open = !self.timer_menu_open,
            Event::CloseTimerMenu => self.timer_menu_open = false,
            Event::SetTimer(timer) => {
                self.timer = timer;
                self.timer_menu_open = false;
            }
            Event::ToggleMirror => self.mirror = !self.mirror,
            Event::PhotoSaved(path) => self.last_capture = Some(CaptureOutcome::PhotoSaved(path)),
            Event::PhotoFailed => self.last_capture = Some(CaptureOutcome::PhotoFailed),
            Event::RecordingStarted => {
                self.video = Some(VideoCapture::Recording(Recording { elapsed_secs: 0 }));
            }
            Event::RecordingTick => {
                if let Some(VideoCapture::Recording(recording)) = &mut self.video {
                    recording.elapsed_secs += 1;
                }
            }
            Event::RecordingStopped if self.recording().is_some() => {
                self.video = Some(VideoCapture::Saving);
            }
            Event::RecordingStopped => {}
            Event::VideoSaved(path) => self.finish_video(CaptureOutcome::VideoSaved(path)),
            Event::VideoFailed => self.finish_video(CaptureOutcome::VideoFailed),
        }
    }

    /// End the Video, whatever its phase, and record how it went.
    fn finish_video(&mut self, outcome: CaptureOutcome) {
        self.video = None;
        self.last_capture = Some(outcome);
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
        assert_eq!(state.last_capture, None);
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

    /// Choosing any Timer value applies it and dismisses its menu.
    #[test]
    fn timer_selection_sets_the_value_and_closes_the_menu() {
        let mut state = AppState::new();
        for timer in [Timer::TenSeconds, Timer::ThreeSeconds, Timer::Off] {
            state.handle(Event::ToggleTimerMenu);
            assert!(state.timer_menu_open);
            state.handle(Event::SetTimer(timer));
            assert_eq!(state.timer, timer);
            assert!(!state.timer_menu_open);
        }
    }

    /// Esc and outside clicks dismiss the menu without altering the selected Timer.
    #[test]
    fn closing_the_timer_menu_preserves_the_selection() {
        let mut state = AppState::new();
        state.handle(Event::SetTimer(Timer::ThreeSeconds));
        state.handle(Event::ToggleTimerMenu);
        state.handle(Event::CloseTimerMenu);
        assert!(!state.timer_menu_open);
        assert_eq!(state.timer, Timer::ThreeSeconds);
        let before = state.clone();
        state.handle(Event::CloseTimerMenu);
        assert_eq!(state, before);
    }

    /// Cycling while the menu is open updates its selected value without dismissing it.
    #[test]
    fn timer_cycles_with_the_menu_open_and_the_button_can_toggle_it_closed() {
        let mut state = AppState::new();
        state.handle(Event::ToggleTimerMenu);
        for timer in [Timer::ThreeSeconds, Timer::TenSeconds, Timer::Off] {
            state.handle(Event::CycleTimer);
            assert_eq!(state.timer, timer);
            assert!(state.timer_menu_open);
        }
        state.handle(Event::ToggleTimerMenu);
        assert!(!state.timer_menu_open);
        assert_eq!(state.timer, Timer::Off);
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
        assert_eq!(state.last_capture, Some(CaptureOutcome::PhotoSaved(path)));
        assert_eq!(camera.photo_folders, [PathBuf::from("/pics/Camera")]);
    }

    #[test]
    fn a_failed_photo_is_recorded_as_failed() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]).failing_photo();
        let mut state = AppState::new();
        state.take_photo(&mut camera, Path::new("/pics/Camera"));
        assert_eq!(state.last_capture, Some(CaptureOutcome::PhotoFailed));
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
        assert_eq!(state.recording(), Some(Recording { elapsed_secs: 0 }));
        assert_eq!(camera.video_folders, [PathBuf::from("/videos/Camera")]);
        assert_eq!(state.shutter_action(), Some(ShutterAction::StopRecording));
    }

    /// A failed Video start produces failure feedback and leaves the Shutter ready to retry.
    #[test]
    fn a_recording_that_fails_to_start_stays_idle_and_reports_the_failure() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]).failing_video();
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        assert_eq!(state.video, None);
        assert_eq!(state.last_capture, Some(CaptureOutcome::VideoFailed));
    }

    /// Stopping ends the recording phase and asks the backend to finalize once.
    #[test]
    fn stopping_tells_the_backend_and_ends_the_recording() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        state.stop_recording(&mut camera);
        assert_eq!(state.recording(), None);
        assert_eq!(camera.videos_stopped, 1);
    }

    /// A delayed EOS keeps the Video saving without success feedback until completion.
    #[test]
    fn a_stopped_video_is_saving_until_eos_completes_then_saved() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)])
            .saving_video_as(Path::new("/videos/Camera/a.mp4"));
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        let mut saving = state.stop_recording(&mut camera).unwrap();
        assert_eq!(state.video, Some(VideoCapture::Saving));
        assert_eq!(saving.try_recv(), Ok(None), "EOS has not completed yet");
        assert_eq!(state.last_capture, None);
        camera.complete_eos();
        state.handle(saving.try_recv().unwrap().unwrap().into());
        assert_eq!(state.video, None);
        assert_eq!(
            state.last_capture,
            Some(CaptureOutcome::VideoSaved(PathBuf::from(
                "/videos/Camera/a.mp4"
            )))
        );
        assert_eq!(state.shutter_action(), Some(ShutterAction::StartRecording));
    }

    /// A Video Mode state whose Video was recorded and stopped, with the fake waiting on its EOS.
    fn saving_state(camera: &mut FakeCamera) -> (AppState, VideoSaving) {
        let mut state = video_state();
        state.start_recording(camera, Path::new("/videos/Camera"));
        let saving = state.stop_recording(camera).unwrap();
        (state, saving)
    }

    /// Failed finalization produces failure feedback and restores the Video Shutter.
    #[test]
    fn a_failed_eos_reports_the_failure_and_returns_to_the_video_shutter() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let (mut state, mut saving) = saving_state(&mut camera);
        camera.fail_eos();
        state.handle(saving.try_recv().unwrap().unwrap().into());
        assert_eq!(state.video, None);
        assert_eq!(state.last_capture, Some(CaptureOutcome::VideoFailed));
        assert_eq!(state.shutter_action(), Some(ShutterAction::StartRecording));
    }

    /// Shutter presses, repeated stops and Mode changes cannot interrupt saving.
    #[test]
    fn the_shutter_and_mode_are_ignored_while_saving() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let (mut state, _saving) = saving_state(&mut camera);
        assert_eq!(state.shutter_action(), None);
        state.handle(Event::SetMode(Mode::Photo));
        assert_eq!(state.mode, Mode::Video);
        assert!(state.stop_recording(&mut camera).is_none());
        assert_eq!(camera.videos_stopped, 1);
        assert_eq!(state.video, Some(VideoCapture::Saving));
    }

    /// Neither capture entry point may replace the Video while its EOS is pending.
    #[test]
    fn capture_requests_are_ignored_while_saving() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let (mut state, mut saving) = saving_state(&mut camera);
        let before = state.clone();

        state.take_photo(&mut camera, Path::new("/pics/Camera"));
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        state.handle(Event::RecordingTick);

        assert_eq!(state, before);
        assert!(camera.photo_folders.is_empty());
        assert_eq!(camera.video_folders.len(), 1);
        assert_eq!(saving.try_recv(), Ok(None));
    }

    #[test]
    fn elapsed_seconds_tick_while_recording() {
        let mut camera = FakeCamera::ready("Cam", vec![res(1280, 720)]);
        let mut state = video_state();
        state.start_recording(&mut camera, Path::new("/videos/Camera"));
        for _ in 0..3 {
            state.handle(Event::RecordingTick);
        }
        assert_eq!(state.recording(), Some(Recording { elapsed_secs: 3 }));
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
