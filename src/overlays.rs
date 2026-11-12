//! Static pieces of the Camera window: the Viewfinder image, the scrims and the camera name.

use std::sync::Arc;

use gpui::{
    FontWeight, IntoElement, ObjectFit, RenderImage, SharedString, div, img, linear_color_stop,
    linear_gradient, prelude::*, px, rgba, svg,
};

use crate::theme::{color, font, size, space};

/// Height of the `scrim_top` gradient behind the top row, in logical px.
const SCRIM_TOP_HEIGHT: f32 = 96.0;
/// Height of the `scrim_bottom` gradient behind the bottom controls, in logical px.
const SCRIM_BOTTOM_HEIGHT: f32 = 200.0;

/// The latest frame, as large as fits the window without distortion, centered.
pub fn viewfinder_image(frame: Arc<RenderImage>) -> impl IntoElement {
    img(frame).size_full().object_fit(ObjectFit::Contain)
}

/// Gradient from `color::SCRIM_TOP` at the top edge to transparent, so the top row stays legible.
pub fn scrim_top() -> impl IntoElement {
    scrim(color::SCRIM_TOP, SCRIM_TOP_HEIGHT, 180.0).top_0()
}

/// Gradient from `color::SCRIM_BOTTOM` at the bottom edge to transparent, so the bottom controls stay legible.
pub fn scrim_bottom() -> impl IntoElement {
    scrim(color::SCRIM_BOTTOM, SCRIM_BOTTOM_HEIGHT, 0.0).bottom_0()
}

/// A full-width band `height` tall fading from `edge_color` to transparent, along `angle` degrees.
fn scrim(edge_color: u32, height: f32, angle: f32) -> gpui::Div {
    // Same color with the alpha byte (lowest, in 0xRRGGBBAA) cleared, so only opacity fades.
    let transparent = edge_color & 0xFFFF_FF00;
    div()
        .absolute()
        .left_0()
        .right_0()
        .h(px(height))
        .bg(linear_gradient(
            angle,
            linear_color_stop(rgba(edge_color), 0.0),
            linear_color_stop(rgba(transparent), 1.0),
        ))
}

/// The `camera` icon and the camera name, top left.
pub fn camera_label(name: SharedString) -> impl IntoElement {
    let text_color = rgba(color::TEXT_SECONDARY);
    div()
        .absolute()
        .top(px(space::S16))
        .left(px(space::S16))
        .h(px(size::CONTROL))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_color)
        .child(
            svg()
                .path("icons/camera.svg")
                .size(px(size::ICON_SM))
                .text_color(text_color),
        )
        .child(name)
}
