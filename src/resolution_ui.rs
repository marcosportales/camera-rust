//! Pure visual pieces of the Resolution control and menu.

use crate::app_core::{controls::ControlLock, resolution::Resolution};
use crate::overlays::shadow_control;
use crate::theme::{color, font, radius, size, space};
use crate::{locked_controls, menu_ui};
use gpui::{Div, FontWeight, Stateful, div, prelude::*, px, rgba, svg};

/// The selected height and chevron, with the shared settings lock explanation.
pub fn button(resolution: Resolution, open: bool, lock: Option<ControlLock>) -> Stateful<Div> {
    let text_color = rgba(if lock.is_some() {
        color::TEXT_DISABLED
    } else {
        color::TEXT_PRIMARY
    });
    let button = div()
        .id("resolution")
        .occlude()
        .h(px(size::CONTROL))
        .px(px(space::S12))
        .flex()
        .items_center()
        .gap(px(space::S4))
        .rounded(px(radius::MD))
        .bg(rgba(if open {
            color::CONTROL_ON
        } else {
            color::CONTROL
        }))
        .shadow(shadow_control())
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(text_color)
        .child(resolution.label())
        .child(
            svg()
                .path("icons/chevron-down.svg")
                .size(px(size::ICON_SM))
                .text_color(text_color),
        );
    match lock {
        Some(lock) => locked_controls::with_lock_explanation(button, lock),
        None => button
            .cursor_pointer()
            .hover(|style| style.bg(rgba(color::CONTROL_HOVER))),
    }
}

/// The 232 px Resolution menu with its caption header and 4 px inset.
pub fn menu(opacity: f32) -> Div {
    menu_ui::surface("Resolution", 232.0, opacity)
}

/// A supported size, including dimensions, aspect ratio and the current selection's check.
pub fn row(resolution: Resolution, selected: Option<Resolution>) -> Stateful<Div> {
    let checked = selected == Some(resolution);
    div()
        .id((
            "resolution-row",
            (u64::from(resolution.width) << 32) | u64::from(resolution.height),
        ))
        .h(px(size::MENU_ROW))
        .px(px(space::S12))
        .flex()
        .items_center()
        .justify_between()
        .rounded(px(radius::SM))
        .text_color(rgba(color::TEXT_PRIMARY))
        .text_size(px(font::LABEL))
        .font_weight(if checked {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::MEDIUM
        })
        .hover(|style| style.bg(rgba(color::ITEM_HOVER)))
        .cursor_pointer()
        .child(description(resolution))
        .when(checked, |row| {
            row.child(
                svg()
                    .path("icons/check.svg")
                    .size(px(size::ICON_MD))
                    .text_color(rgba(color::TEXT_PRIMARY)),
            )
        })
}

/// Pair the height label with the secondary dimensions and aspect ratio.
fn description(resolution: Resolution) -> Div {
    div()
        .flex()
        .items_baseline()
        .gap(px(space::S8))
        .child(resolution.label())
        .child(
            div()
                .text_size(px(font::SMALL))
                .font_weight(FontWeight::NORMAL)
                .text_color(rgba(color::TEXT_TERTIARY))
                .child(resolution.description()),
        )
}
