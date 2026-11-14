//! Schedule idle wakeups and connect core visibility to the overlay controls.
use super::CameraWindow;
use crate::{
    app_core::{auto_hide::AutoHide, state::AppState},
    controls_ui,
};
use gpui::{Context, IntoElement, Task, Window, prelude::*};
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

    /// Apply core visibility and own the task matching its next idle deadline.
    fn sync(&mut self, state: &AppState, movement: bool, cx: &mut Context<CameraWindow>) {
        let now = self.origin.elapsed();
        self.core.update(now, state, movement);
        let deadline = self.core.deadline();
        if self.scheduled != deadline {
            self.scheduled = deadline;
            self.task = deadline.map(|deadline| {
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
}

impl CameraWindow {
    /// Suppress shortcut tooltips as soon as controls stop accepting interaction.
    pub(super) fn shortcut_hint(
        &self,
        control: crate::app_core::tooltips::Control,
    ) -> Option<crate::app_core::tooltips::Hint> {
        self.controls_visibility
            .core
            .interactive()
            .then(|| self.state.shortcut_hint(control))
            .flatten()
    }

    /// Connect mouse or state changes to the owned visibility clock.
    pub(super) fn sync_controls(&mut self, movement: bool, cx: &mut Context<Self>) {
        self.controls_visibility.sync(&self.state, movement, cx);
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
            .child(self.thumbnail(window, cx))
            .child(crate::mirror_ui::control(
                &self.state,
                self.shortcut_hint(crate::app_core::tooltips::Control::Mirror),
                cx.listener(|this, _, window, cx| this.toggle_mirror(window, cx)),
            ))
            .child(self.mode_pill(cx))
            .child(self.settings_controls(window, cx));
        controls_ui::surface(
            layer,
            opacity,
            hidden,
            cx.listener(|this, _, _, cx| {
                this.sync_controls(true, cx);
                cx.stop_propagation();
                cx.notify();
            }),
            cx.listener(|this, _, _, cx| {
                this.sync_controls(true, cx);
                cx.notify();
            }),
        )
    }
}
