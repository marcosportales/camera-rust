//! Wiring between Timer controls and the app core.

use gpui::{Context, IntoElement, Window, div, prelude::*};

use super::CameraWindow;
use crate::app_core::state::{Event, Timer};
use crate::timer_ui;

impl CameraWindow {
    /// Apply a Timer event and fade its menu if its visibility changed.
    pub(super) fn timer_event(&mut self, event: Event, cx: &mut Context<Self>) {
        let was_open = self.state.timer_menu_open;
        self.state.handle(event);
        if was_open != self.state.timer_menu_open {
            self.timer_menu_fade.retarget(self.state.timer_menu_open);
        }
        cx.notify();
    }

    /// The Timer and its floating menu; outside presses dismiss and consume the click.
    pub(super) fn timer_controls(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.timer_menu_fade.animating() {
            window.request_animation_frame();
        }
        let opacity = self.timer_menu_fade.opacity();
        let control = div()
            .relative()
            .child(
                timer_ui::button(self.state.timer).on_click(cx.listener(|this, _, _, cx| {
                    this.timer_event(Event::ToggleTimerMenu, cx);
                })),
            )
            .when(self.state.timer_menu_open || opacity > 0.0, |control| {
                control.child(self.timer_menu(opacity, cx))
            });
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .when(self.state.timer_menu_open, |layer| {
                layer.child(timer_ui::dismiss_layer(cx.listener(|this, _, _, cx| {
                    this.timer_event(Event::CloseTimerMenu, cx);
                    cx.stop_propagation();
                })))
            })
            .child(timer_ui::controls().child(control))
    }

    /// The current Timer choices; rows stop accepting clicks as soon as the menu starts closing.
    fn timer_menu(&self, opacity: f32, cx: &mut Context<Self>) -> impl IntoElement {
        timer_ui::menu(opacity)
            .children(
                [Timer::Off, Timer::ThreeSeconds, Timer::TenSeconds].map(|timer| {
                    timer_ui::row(timer, self.state.timer).when(self.state.timer_menu_open, |row| {
                        row.on_click(cx.listener(move |this, _, _, cx| {
                            this.timer_event(Event::SetTimer(timer), cx);
                        }))
                    })
                }),
            )
            .child(timer_ui::shortcut_hint())
    }
}
