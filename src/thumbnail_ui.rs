//! Last-Capture Thumbnail visuals, independent of the window and capture IO.

use std::path::Path;

use gpui::{
    App, ClickEvent, Div, IntoElement, ObjectFit, Stateful, Window, div, img, prelude::*, px,
    relative, rgba, svg,
};

use crate::theme::{color, radius, size, space};

/// Draw the Thumbnail 40 px left of the centered Shutter, with Photo or empty visuals.
pub fn control(
    photo: Option<&Path>,
    just_saved: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .right(relative(0.5))
        .mr(px(size::SHUTTER / 2.0 + space::S40))
        .bottom(px(space::S24 + (size::SHUTTER - size::THUMB) / 2.0))
        .child(button(photo, just_saved).on_click(on_click))
}

/// Style the clipped Photo, dashed empty state, hover border and just-saved border.
fn button(photo: Option<&Path>, just_saved: bool) -> Stateful<Div> {
    div()
        .id("last-capture-thumbnail")
        .size(px(size::THUMB))
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .overflow_hidden()
        .cursor_pointer()
        .bg(rgba(color::CONTROL))
        .border_color(rgba(if just_saved {
            color::SHUTTER_RING
        } else {
            color::BORDER_STRONG
        }))
        .border(px(if just_saved { 2.0 } else { 1.5 }))
        .when(!just_saved, |button| {
            button.hover(|style| style.border_color(rgba(color::BORDER_STRONG_HOVER)))
        })
        .when(photo.is_none(), |button| {
            button.border_dashed().child(
                svg()
                    .path("icons/folder.svg")
                    .size(px(size::ICON_MD))
                    .text_color(rgba(color::TEXT_TERTIARY)),
            )
        })
        .children(photo.map(|path| {
            img(path.to_path_buf())
                .size_full()
                .rounded(px(radius::MD - 1.5))
                .object_fit(ObjectFit::Cover)
        }))
}
