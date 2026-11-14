//! Schedule idle wakeups and connect core visibility to the overlay controls.
use super::CameraWindow;
use crate::{app_core::auto_hide::AutoHide, controls_ui};
use gpui::{Context, IntoElement, Task, Window, div, prelude::*};
use std::time::{Duration, Instant};

/// Monotonic clock, visibility state and the single pending idle wakeup.
pub(super) struct ControlsVisibility {
    origin: Instant,
    core: AutoHide,
    scheduled: Option<Duration>,
    task: Option<Task<()>>,
}

impl ControlsVisibility {
    /// Begin visible with an idle timer relative to this window's creation.
    pub(super) fn new() -> Self {
        Self {
            origin: Instant::now(),
            core: AutoHide::new(),
            scheduled: None,
            task: None,
        }
    }
}

impl CameraWindow {
    /// Apply mouse or state changes and replace the wakeup only when its deadline changes.
    pub(super) fn sync_controls(&mut self, movement: bool, cx: &mut Context<Self>) {
        let controls = &mut self.controls_visibility;
        let now = controls.origin.elapsed();
        controls.core.update(now, &self.state, movement);
        let deadline = controls.core.deadline();
        if controls.scheduled != deadline {
            controls.scheduled = deadline;
            controls.task = deadline.map(|deadline| {
                cx.spawn(async move |this, cx| {
                    cx.background_executor()
                        .timer(deadline.saturating_sub(now))
                        .await;
                    let _ = this.update(cx, |view, cx| {
                        view.sync_controls(false, cx);
                        cx.notify();
                    });
                })
            });
        }
    }

    /// Fade all controls together and shield them from the mouse as soon as hiding begins.
    pub(super) fn overlay_controls(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let controls = &self.controls_visibility;
        let now = controls.origin.elapsed();
        let opacity = controls.core.opacity(now);
        if controls.core.animating(now) {
            window.request_animation_frame();
        }
        let hidden = !controls.core.interactive();
        let layer = controls_ui::layer(opacity)
            .child(crate::overlays::scrim_top())
            .child(crate::overlays::scrim_bottom())
            .child(self.shutter(cx))
            .child(crate::mirror_ui::control(
                &self.state,
                cx.listener(|this, _, window, cx| this.toggle_mirror(window, cx)),
            ))
            .child(self.mode_pill(cx))
            .child(self.settings_controls(window, cx));
        div()
            .absolute()
            .size_full()
            .when(opacity > 0.0, |root| root.child(layer))
            .when(hidden, |root| {
                root.child(
                    div()
                        .id("wake-controls")
                        .absolute()
                        .size_full()
                        .occlude()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sync_controls(true, cx);
                            cx.stop_propagation();
                            cx.notify();
                        })),
                )
            })
            .child(controls_ui::movement_observer(cx.listener(
                |this, _, _, cx| {
                    this.sync_controls(true, cx);
                    cx.notify();
                },
            )))
    }
}
