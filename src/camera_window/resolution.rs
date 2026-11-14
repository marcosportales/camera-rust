//! Glue between Resolution controls, the app core and the camera backend.

use super::CameraWindow;
use crate::app_core::{resolution::Resolution, state::Event};
use crate::resolution_ui;
use gpui::{Context, IntoElement, div, prelude::*};

impl CameraWindow {
    /// Apply the core's restart and rollback, then fade the closed menu.
    fn select_resolution(&mut self, resolution: Resolution, cx: &mut Context<Self>) {
        if let Err(error) = self.state.select_resolution(resolution, &mut self.camera) {
            eprintln!("Could not switch Resolution: {error:?}");
        }
        self.resolution_menu_fade
            .retarget(self.state.resolution_menu_open);
        cx.notify();
    }

    /// The Resolution button and floating choices, next to the Timer.
    pub(super) fn resolution_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(resolution) = self.state.resolution else {
            return div();
        };
        let lock = self.state.control_lock();
        let open = self.state.resolution_menu_open;
        let opacity = self.resolution_menu_fade.opacity();
        div()
            .relative()
            .child(
                resolution_ui::button(resolution, open, lock).when(lock.is_none(), |button| {
                    button.on_click(cx.listener(|this, _, _, cx| {
                        this.settings_event(Event::ToggleResolutionMenu, cx);
                    }))
                }),
            )
            .when(lock.is_none() && (open || opacity > 0.0), |control| {
                control.child(resolution_ui::menu(opacity).children(
                    self.state.resolution_choices().into_iter().map(|choice| {
                        resolution_ui::row(choice, self.state.resolution).when(open, |row| {
                            row.on_click(cx.listener(move |this, _, _, cx| {
                                this.select_resolution(choice, cx);
                            }))
                        })
                    }),
                ))
            })
    }
}
