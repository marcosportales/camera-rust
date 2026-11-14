//! Schedule Capture Timer updates; dropping the task cancels pending wakeups.
use super::CameraWindow;
use crate::app_core::state::ShutterAction;
use gpui::{Context, Task};
use std::time::{Duration, Instant};

/// The monotonic origin and owned task for this window's Capture Timer.
pub(super) struct CountdownClock {
    origin: Instant,
    task: Option<Task<()>>,
}

impl CountdownClock {
    /// Start a clock with no scheduled countdown.
    pub(super) fn new() -> Self {
        Self {
            origin: Instant::now(),
            task: None,
        }
    }

    /// Read time relative to this window's monotonic origin.
    pub(super) fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

impl CameraWindow {
    /// Cancel the core countdown and its pending wakeup without feedback.
    pub(super) fn cancel_countdown(&mut self, cx: &mut Context<Self>) {
        self.state.cancel_countdown();
        self.countdown_clock.task = None;
        cx.notify();
    }

    /// Replace the scheduled task to match the active core countdown.
    pub(super) fn sync_countdown(&mut self, cx: &mut Context<Self>) {
        self.countdown_clock.task = None;
        self.timer_menu_fade.retarget(self.state.timer_menu_open);
        if self.state.countdown_numeral().is_some() {
            self.countdown_clock.task = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_secs(1)).await;
                    let running = this.update(cx, |view, cx| {
                        let action = view.state.advance_countdown(view.countdown_clock.now());
                        match action {
                            Some(ShutterAction::TakePhoto) => view.take_photo(cx),
                            Some(ShutterAction::StartRecording) => view.start_recording(cx),
                            _ => {}
                        }
                        cx.notify();
                        view.state.countdown_numeral().is_some()
                    });
                    if !matches!(running, Ok(true)) {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
}
