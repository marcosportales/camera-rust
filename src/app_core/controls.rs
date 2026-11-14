//! Shared availability of capture settings in the core and view.

use super::state::{AppState, Event};

/// The capture phase that prevents changing settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlLock {
    Countdown,
    Recording,
}

impl ControlLock {
    /// Explain why a settings control is unavailable, without a shortcut hint.
    pub fn message(self) -> &'static str {
        match self {
            Self::Countdown => "Unavailable during countdown",
            Self::Recording => "Unavailable while recording",
        }
    }
}

impl AppState {
    /// The lock shared by settings events and their visual controls.
    pub fn control_lock(&self) -> Option<ControlLock> {
        if self.countdown.is_some() {
            Some(ControlLock::Countdown)
        } else if self.recording().is_some() {
            Some(ControlLock::Recording)
        } else {
            None
        }
    }

    /// Reject settings changes while a countdown or recording is active.
    pub(super) fn ignores_locked_control(&self, event: &Event) -> bool {
        self.control_lock().is_some()
            && matches!(
                event,
                Event::SetMode(_)
                    | Event::CycleTimer
                    | Event::ToggleTimerMenu
                    | Event::SetTimer(_)
                    | Event::ToggleMirror
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::app_core::state::{AppState, Event, Mode, ShutterAction, Timer};
    use std::time::Duration;

    /// Every existing settings event is ignored throughout countdown and recording.
    #[test]
    fn locked_controls_ignore_settings_events() {
        for start in [
            Event::StartCountdown {
                action: ShutterAction::TakePhoto,
                now: Duration::ZERO,
                seconds: 3,
            },
            Event::RecordingStarted,
        ] {
            let mut state = AppState::new();
            state.handle(start);
            for event in [
                Event::SetMode(Mode::Video),
                Event::SetMode(Mode::Photo),
                Event::CycleTimer,
                Event::ToggleTimerMenu,
                Event::SetTimer(Timer::ThreeSeconds),
                Event::SetTimer(Timer::TenSeconds),
                Event::SetTimer(Timer::Off),
                Event::ToggleMirror,
            ] {
                let before = state.clone();
                state.handle(event.clone());
                assert_eq!(state, before, "locked event: {event:?}");
            }
        }
    }

    /// Cancellation, a completed Photo countdown and a finished Video restore settings.
    #[test]
    fn locked_controls_unlock_after_capture_or_cancellation() {
        let countdown = Event::StartCountdown {
            action: ShutterAction::TakePhoto,
            now: Duration::ZERO,
            seconds: 3,
        };
        let mut cancelled = AppState::new();
        cancelled.handle(countdown.clone());
        cancelled.cancel_countdown();
        let mut photo = AppState::new();
        photo.handle(countdown);
        assert_eq!(
            photo.advance_countdown(Duration::from_secs(3)),
            Some(ShutterAction::TakePhoto)
        );
        photo.handle(Event::PhotoSaved("photo.jpg".into()));
        let mut video = AppState::new();
        video.handle(Event::RecordingStarted);
        video.handle(Event::RecordingStopped);
        let saving = video.clone();
        video.handle(Event::VideoSaved("video.mp4".into()));
        for mut state in [AppState::new(), cancelled, photo, saving, video] {
            assert_eq!(state.control_lock(), None);
            state.handle(Event::SetMode(Mode::Video));
            assert_eq!(state.mode, Mode::Video);
            state.handle(Event::SetMode(Mode::Photo));
            assert_eq!(state.mode, Mode::Photo);
            state.handle(Event::ToggleTimerMenu);
            assert!(state.timer_menu_open);
            state.handle(Event::SetTimer(Timer::ThreeSeconds));
            assert_eq!(state.timer, Timer::ThreeSeconds);
            assert!(!state.timer_menu_open);
            state.handle(Event::CycleTimer);
            assert_eq!(state.timer, Timer::TenSeconds);
            state.handle(Event::ToggleMirror);
            assert!(!state.mirror);
        }
    }

    /// Starting either capture phase dismisses a menu and provides the matching explanation.
    #[test]
    fn locked_controls_close_the_menu_and_explain_the_phase() {
        use super::ControlLock;
        let mut state = AppState::new();
        state.handle(Event::ToggleTimerMenu);
        state.handle(Event::StartCountdown {
            action: ShutterAction::StartRecording,
            now: Duration::ZERO,
            seconds: 3,
        });
        assert!(!state.timer_menu_open);
        assert_eq!(state.control_lock(), Some(ControlLock::Countdown));
        assert_eq!(
            state.control_lock().unwrap().message(),
            "Unavailable during countdown"
        );
        state.cancel_countdown();
        state.handle(Event::ToggleTimerMenu);
        state.handle(Event::RecordingStarted);
        assert!(!state.timer_menu_open);
        assert_eq!(state.control_lock(), Some(ControlLock::Recording));
        assert_eq!(
            state.control_lock().unwrap().message(),
            "Unavailable while recording"
        );
        state.handle(Event::RecordingTick);
        assert_eq!(state.recording().unwrap().elapsed_secs, 1);
    }
}
