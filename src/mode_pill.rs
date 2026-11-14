//! The Mode pill: two segments, Photo and Video, above the Shutter.

use gpui::{
    Animation, AnimationExt, App, ClickEvent, Div, FontWeight, IntoElement, Stateful, Window, div,
    prelude::*, px, rgba,
};

use crate::app_core::controls::ControlLock;
use crate::app_core::state::Mode;
use crate::locked_controls;
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
    lock: Option<ControlLock>,
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
    let (from, to) = segment_colors(target, selected, previous);
    let button: Stateful<Div> = div()
        .id(id)
        .h(px(size::SEGMENT))
        .px(px(space::S16))
        .flex()
        .items_center()
        .rounded(px(radius::FULL))
        .text_size(px(font::LABEL));
    let button = if let Some(lock) = lock {
        locked_controls::with_lock_explanation(button, lock)
    } else {
        button.on_click(on_select)
    };
    let button = segment_appearance(button, active, lock.is_some());
    button.child(label).with_animation(
        (id, epoch),
        Animation::new(motion::MODE_SWITCH),
        move |segment, progress| segment.bg(mix(from, to, progress)),
    )
}

/// Fade only when this segment changed selection; otherwise hold its resting fill.
fn segment_colors(target: Mode, selected: Mode, previous: Mode) -> (u32, u32) {
    let fill = |mode| {
        if mode == target {
            color::SELECTED
        } else {
            SEGMENT_CLEAR
        }
    };
    (fill(previous), fill(selected))
}

/// Apply selected or inactive text styling, suppressing hover while locked.
fn segment_appearance(button: Stateful<Div>, active: bool, locked: bool) -> Stateful<Div> {
    if locked {
        button.font_weight(FontWeight::MEDIUM)
    } else if active {
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
    }
}
