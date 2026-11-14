//! The Mirror control beside the centered Shutter.

use gpui::{
    App, ClickEvent, Div, IntoElement, Stateful, Window, div, prelude::*, px, relative, rgba, svg,
};

use crate::app_core::state::AppState;
use crate::locked_controls;
use crate::overlays::shadow_control;
use crate::theme::{color, radius, size, space};

/// Draw the 48 px Mirror button 40 px right of the Shutter, preserving existing control locks.
pub fn control(
    state: &AppState,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let lock = state.control_lock();
    let button = button(state);
    let button = match lock {
        Some(lock) => locked_controls::with_lock_explanation(button, lock),
        None => button.cursor_pointer().on_click(on_click),
    };
    div()
        .absolute()
        .left(relative(0.5))
        .ml(px(size::SHUTTER / 2.0 + space::S40))
        .bottom(px(space::S24 + (size::SHUTTER - size::MIRROR) / 2.0))
        .child(button)
}

/// Style the Mirror button and icon with the current value and availability.
fn button(state: &AppState) -> Stateful<Div> {
    let lock = state.control_lock();
    div()
        .id("mirror")
        .size(px(size::MIRROR))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::FULL))
        .bg(rgba(if state.mirror {
            color::CONTROL_ON
        } else {
            color::CONTROL_HOVER
        }))
        .shadow(shadow_control())
        .child(
            svg()
                .path("icons/flip-horizontal.svg")
                .size(px(size::ICON_LG))
                .text_color(rgba(if lock.is_some() {
                    color::TEXT_DISABLED
                } else {
                    color::TEXT_PRIMARY
                })),
        )
}
