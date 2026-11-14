//! Photo Timer transitions, driven by an injected monotonic clock.

use std::time::Duration;

use super::state::{AppState, Event, ShutterAction, Timer};

/// An active Photo Timer and its last displayed whole second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Countdown {
    pub(super) deadline: Duration,
    pub(super) remaining: u8,
}

impl AppState {
    /// Start a Photo Timer, cancel one in progress, or return the immediate capture action.
    pub fn press_shutter(&mut self, now: Duration) -> Option<ShutterAction> {
        if self.countdown.is_some() {
            self.cancel_countdown();
            return None;
        }
        let action = self.shutter_action()?;
        let seconds = match (action, self.timer) {
            (ShutterAction::TakePhoto, Timer::ThreeSeconds) => 3,
            (ShutterAction::TakePhoto, Timer::TenSeconds) => 10,
            _ => return Some(action),
        };
        self.handle(Event::StartCountdown { now, seconds });
        None
    }

    /// Update the numeral from monotonic time and emit the capture once at the deadline.
    pub fn advance_countdown(&mut self, now: Duration) -> Option<ShutterAction> {
        let countdown = self.countdown?;
        if now >= countdown.deadline {
            self.cancel_countdown();
            return Some(ShutterAction::TakePhoto);
        }
        self.handle(Event::CountdownTick(now));
        None
    }

    /// The visible numeral, absent when no Photo Timer is active.
    pub fn countdown_numeral(&self) -> Option<u8> {
        self.countdown.map(|countdown| countdown.remaining)
    }

    /// Cancel without producing a capture or changing feedback.
    pub fn cancel_countdown(&mut self) {
        self.handle(Event::CancelCountdown);
    }
}

#[cfg(test)]
mod tests {
    use crate::app_core::{
        backend::Camera,
        state::{AppState, Event, ShutterAction, Timer},
    };
    use std::time::Duration;

    /// Three seconds display 3, 2, 1 and yield exactly one Photo.
    #[test]
    fn countdown_finishes_once() {
        let mut state = AppState::new();
        state.handle(Event::CameraReady(Camera {
            name: "Cam".into(),
            resolutions: vec![],
        }));
        state.handle(Event::SetTimer(Timer::ThreeSeconds));
        assert_eq!(state.press_shutter(Duration::ZERO), None);
        for (second, numeral) in [(0, 3), (1, 2), (2, 1)] {
            assert_eq!(state.advance_countdown(Duration::from_secs(second)), None);
            assert_eq!(state.countdown_numeral(), Some(numeral));
        }
        assert_eq!(
            state.advance_countdown(Duration::from_secs(3)),
            Some(ShutterAction::TakePhoto)
        );
        assert_eq!(state.countdown_numeral(), None);
        assert_eq!(state.advance_countdown(Duration::from_secs(4)), None);
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use crate::app_core::{backend::Camera, state::Event};

    /// A ready Photo state with the selected Timer.
    fn ready(timer: Timer) -> AppState {
        let mut state = AppState::new();
        state.handle(Event::CameraReady(Camera {
            name: "Cam".into(),
            resolutions: vec![],
        }));
        state.handle(Event::SetTimer(timer));
        state
    }

    /// Both cancellation routes suppress capture at every displayed second.
    #[test]
    fn countdown_cancels_at_every_tick() {
        for timer in [Timer::ThreeSeconds, Timer::TenSeconds] {
            let seconds = if timer == Timer::ThreeSeconds { 3 } else { 10 };
            for tick in 0..seconds {
                for shutter in [false, true] {
                    let mut state = ready(timer);
                    state.press_shutter(Duration::ZERO);
                    state.advance_countdown(Duration::from_secs(tick));
                    if shutter {
                        assert_eq!(state.press_shutter(Duration::from_secs(tick)), None);
                    } else {
                        state.cancel_countdown();
                    }
                    assert_eq!(state.countdown_numeral(), None);
                    assert_eq!(state.advance_countdown(Duration::from_secs(20)), None);
                    assert_eq!(state.last_capture, None);
                }
            }
        }
    }

    /// Ten seconds count down fully; subsecond updates never advance early.
    #[test]
    fn countdown_ten_seconds_and_immediate_off() {
        let mut state = ready(Timer::Off);
        assert_eq!(
            state.press_shutter(Duration::ZERO),
            Some(ShutterAction::TakePhoto)
        );
        state.handle(Event::SetTimer(Timer::TenSeconds));
        state.press_shutter(Duration::ZERO);
        for second in 0..10 {
            assert_eq!(
                state.advance_countdown(Duration::from_millis(second * 1000 + 999)),
                None
            );
            assert_eq!(state.countdown_numeral(), Some((10 - second) as u8));
        }
        assert_eq!(
            state.advance_countdown(Duration::from_secs(10)),
            Some(ShutterAction::TakePhoto)
        );
    }
}
