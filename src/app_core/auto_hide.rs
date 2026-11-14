//! Controls visibility driven by a caller-supplied monotonic clock.
use super::state::AppState;
use crate::theme::motion;
use std::time::Duration;

/// Idle deadline and reversible opacity transition, independent of the UI clock.
pub struct AutoHide {
    deadline: Option<Duration>,
    paused: bool,
    visible: bool,
    from: f32,
    started: Duration,
}

impl AutoHide {
    /// Start with visible controls and the first idle deadline.
    pub fn new() -> Self {
        Self {
            deadline: Some(motion::IDLE_TIMEOUT),
            paused: false,
            visible: true,
            from: 1.0,
            started: Duration::ZERO,
        }
    }

    /// Apply mouse activity and the capture/menu state at `now`.
    pub fn update(&mut self, now: Duration, state: &AppState, movement: bool) {
        let paused = state.timer_menu_open
            || state.resolution_menu_open
            || state.countdown.is_some()
            || state.video.is_some();
        if paused {
            self.visible = true;
            self.from = 1.0;
            self.deadline = None;
        } else if movement || self.paused {
            self.deadline = Some(now + motion::IDLE_TIMEOUT);
            self.retarget(now, true);
        } else if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.deadline = None;
            self.retarget(now, false);
        }
        self.paused = paused;
    }

    /// Preserve the current opacity when changing fade direction.
    fn retarget(&mut self, now: Duration, visible: bool) {
        if self.visible != visible {
            self.from = self.opacity(now);
            self.started = now;
            self.visible = visible;
        }
    }

    /// Whether controls can receive mouse interaction.
    pub fn interactive(&self) -> bool {
        self.visible
    }

    /// The next idle wakeup, absent while paused or hidden.
    pub fn deadline(&self) -> Option<Duration> {
        self.deadline
    }

    /// Current opacity with ease-out on entry and ease-in on exit.
    pub fn opacity(&self, now: Duration) -> f32 {
        let duration = if self.visible {
            motion::CONTROLS_IN
        } else {
            motion::CONTROLS_OUT
        };
        let t = (now.saturating_sub(self.started).as_secs_f32() / duration.as_secs_f32()).min(1.0);
        let eased = if self.visible {
            1.0 - (1.0 - t).powi(2)
        } else {
            t * t
        };
        let target = if self.visible { 1.0 } else { 0.0 };
        self.from + (target - self.from) * eased
    }

    /// Whether the renderer needs another animation frame.
    pub fn animating(&self, now: Duration) -> bool {
        let target = if self.visible { 1.0 } else { 0.0 };
        self.opacity(now) != target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Idle controls begin fading at exactly 2500 ms.
    #[test]
    fn auto_hide_after_idle_timeout() {
        let state = AppState::new();
        let mut controls = AutoHide::new();
        controls.update(Duration::from_millis(2499), &state, false);
        assert!(controls.interactive());
        controls.update(Duration::from_millis(2500), &state, false);
        assert!(!controls.interactive());
        assert_eq!(controls.opacity(Duration::from_millis(2800)), 0.0);
    }
    /// Movement reverses a fade continuously and starts a fresh idle interval.
    #[test]
    fn auto_hide_wakes_on_movement() {
        let state = AppState::new();
        let mut controls = AutoHide::new();
        controls.update(Duration::from_millis(2500), &state, false);
        assert!((controls.opacity(Duration::from_millis(2650)) - 0.75).abs() < 0.001);
        controls.update(Duration::from_millis(2650), &state, true);
        assert!(controls.interactive());
        assert_eq!(controls.opacity(Duration::from_millis(2650)), 0.75);
        assert_eq!(controls.opacity(Duration::from_millis(2800)), 1.0);
        controls.update(Duration::from_millis(5149), &state, false);
        assert!(controls.interactive());
        controls.update(Duration::from_millis(5150), &state, false);
        assert!(!controls.interactive());
    }

    /// Menus and all capture phases force full visibility and restart idle on exit.
    #[test]
    fn auto_hide_pauses_for_menus_countdown_recording_and_saving() {
        use super::super::state::{Event, ShutterAction};
        for event in [
            Event::ToggleTimerMenu,
            Event::ToggleResolutionMenu,
            Event::StartCountdown {
                action: ShutterAction::TakePhoto,
                now: Duration::ZERO,
                seconds: 3,
            },
            Event::RecordingStarted,
            Event::RecordingStopped,
        ] {
            let mut state = AppState::new();
            let mut controls = AutoHide::new();
            controls.update(Duration::from_millis(2500), &state, false);
            if event == Event::RecordingStopped {
                state.handle(Event::RecordingStarted);
            }
            state.handle(event);
            controls.update(Duration::from_secs(4), &state, false);
            controls.update(Duration::from_secs(60), &state, false);
            assert!(controls.interactive());
            assert_eq!(controls.opacity(Duration::from_secs(60)), 1.0);
            assert_eq!(controls.deadline(), None);
            state.handle(Event::CloseTimerMenu);
            state.handle(Event::CloseResolutionMenu);
            state.handle(Event::CancelCountdown);
            state.handle(Event::VideoFailed);
            controls.update(Duration::from_secs(60), &state, false);
            controls.update(Duration::from_millis(62499), &state, false);
            assert!(controls.interactive());
            controls.update(Duration::from_millis(62500), &state, false);
            assert!(!controls.interactive());
        }
    }
}
