//! Last-Capture Thumbnail visuals, independent of the window and capture IO.

mod capture;
mod video;

use std::{path::Path, sync::Arc};

use gpui::{
    App, ClickEvent, Div, IntoElement, RenderImage, Stateful, Window, div, img, prelude::*, px,
    relative, rgba, svg,
};

use crate::theme::{color, radius, size, space};

/// Fetch a cached Capture without blocking rendering, redrawing once decoding finishes.
pub fn image(path: Option<&Path>, window: &mut Window, cx: &mut App) -> Option<Arc<RenderImage>> {
    path.and_then(|path| {
        window
            .use_asset::<capture::SquareCapture>(&path.to_path_buf(), cx)
            .flatten()
    })
}

/// Draw the Thumbnail 40 px left of the centered Shutter, with Capture or empty visuals.
pub fn control(
    capture: Option<&Path>,
    just_saved: bool,
    image: Option<Arc<RenderImage>>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let video = image.is_some()
        && capture.is_some_and(|path| path.extension().is_some_and(|ext| ext == "mp4"));
    div()
        .absolute()
        .right(relative(0.5))
        .mr(px(size::SHUTTER / 2.0 + space::S40))
        .bottom(px(space::S24 + (size::SHUTTER - size::THUMB) / 2.0))
        .child(button(image, video, just_saved).on_click(on_click))
}

/// Style the clipped Capture, dashed empty state, hover border and just-saved border.
fn button(image: Option<Arc<RenderImage>>, video: bool, just_saved: bool) -> Stateful<Div> {
    let just_saved = just_saved && image.is_some();
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
        .when(image.is_none(), |button| {
            button.border_dashed().child(
                svg()
                    .path("icons/folder.svg")
                    .size(px(size::ICON_MD))
                    .text_color(rgba(color::TEXT_TERTIARY)),
            )
        })
        .children(image.map(|image| img(image).size_full().rounded(px(radius::MD - 1.5))))
        .when(video, |button| button.child(video::badge()))
}
