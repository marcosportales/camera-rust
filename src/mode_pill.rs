//! The Mode pill: two segments, Photo and Video, above the Shutter.

use gpui::{
    Animation, AnimationExt, App, ClickEvent, Div, FontWeight, IntoElement, Stateful, Window, div,
    prelude::*, px, rgba,
};

use crate::app_core::state::Mode;
use crate::overlays::shadow_control;
use crate::photo_ui::mix;
use crate::theme::{color, font, motion, radius, size, space};

/// Fill of an inactive segment: the selected color with zero opacity, so only opacity fades.
const SEGMENT_CLEAR: u32 = color::SELECTED & 0xFFFF_FF00;

/// The pill track holding the segments.
pub fn track() -> Div {
    div()
        .p(px(space::S2))
        .flex()
        .gap(px(space::S2))
        .rounded(px(radius::FULL))
        .bg(rgba(color::CONTROL))
        .shadow(shadow_control())
}

/// The segment selecting `target`. `selected` is the current Mode and `previous` the one before it;
/// a new `epoch` fades the fill from `previous` to `selected`.
pub fn segment<F>(
    target: Mode,
    selected: Mode,
    previous: Mode,
    epoch: usize,
    on_select: F,
) -> impl IntoElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let (id, label) = match target {
        Mode::Photo => ("mode-photo", "Photo"),
        Mode::Video => ("mode-video", "Video"),
    };
    let active = selected == target;
    // Fade only when this segment's state changed; otherwise hold the resting fill.
    let resting = if active {
        color::SELECTED
    } else {
        SEGMENT_CLEAR
    };
    let from = if previous == selected {
        resting
    } else if active {
        SEGMENT_CLEAR
    } else {
        color::SELECTED
    };
    let to = resting;
    let button: Stateful<Div> = div()
        .id(id)
        .h(px(size::SEGMENT))
        .px(px(space::S16))
        .flex()
        .items_center()
        .rounded(px(radius::FULL))
        .text_size(px(font::LABEL))
        .on_click(on_select);
    let button = if active {
        button
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgba(color::TEXT_ON_SELECTED))
    } else {
        button
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgba(color::TEXT_SECONDARY))
            .hover(|style| {
                style
                    .bg(rgba(color::ITEM_HOVER))
                    .text_color(rgba(color::TEXT_PRIMARY))
            })
    };
    button.child(label).with_animation(
        (id, epoch),
        Animation::new(motion::MODE_SWITCH),
        move |segment, progress| segment.bg(mix(from, to, progress)),
    )
}
