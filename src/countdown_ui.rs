//! Pure visuals for the Photo Timer.
use crate::theme::{color, font, motion, radius, size, space};
use gpui::{
    Animation, AnimationExt, Font, FontFeatures, FontWeight, IntoElement, div, prelude::*, px,
    rgba, svg,
};
use std::sync::Arc;

/// Dim the Viewfinder during countdown, otherwise use the ready screen top scrim.
pub fn backdrop(active: bool) -> impl IntoElement {
    div()
        .absolute()
        .size_full()
        .when(active, |layer| layer.bg(rgba(color::DIM)))
        .when(!active, |layer| layer.child(crate::overlays::scrim_top()))
}

/// Display a tabular numeral and its cancellation hint at the design's 112px top offset.
pub fn countdown(numeral: u8) -> impl IntoElement {
    let number = div()
        .font(Font {
            family: font::FAMILY.into(),
            features: FontFeatures(Arc::new(vec![("tnum".into(), 1)])),
            weight: FontWeight::LIGHT,
            fallbacks: None,
            style: gpui::FontStyle::Normal,
        })
        .text_size(px(font::COUNTDOWN))
        .line_height(px(font::COUNTDOWN))
        .text_color(rgba(color::TEXT_PRIMARY))
        .child(numeral.to_string())
        .with_animation(
            ("countdown", numeral as u64),
            Animation::new(motion::TICK_FADE),
            |number, progress| number.opacity(progress),
        );
    div()
        .absolute()
        .top(px(112.0))
        .left_0()
        .right_0()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(space::S16))
        .child(number)
        .child(hint())
}

/// The small Esc keycap and cancellation hint below the numeral.
fn hint() -> impl IntoElement {
    let key = div()
        .h(px(size::KEYCAP))
        .px(px(6.0))
        .rounded_full()
        .border_1()
        .border_color(rgba(color::KEYCAP_BORDER))
        .bg(rgba(color::KEYCAP))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(font::CAPTION))
        .font_weight(FontWeight::SEMIBOLD)
        .child("Esc");
    div()
        .h(px(size::TOOLTIP))
        .pl(px(space::S4))
        .pr(px(space::S12))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .rounded(px(radius::FULL))
        .bg(rgba(color::CONTROL))
        .text_color(rgba(color::TEXT_SECONDARY))
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::MEDIUM)
        .child(key)
        .child("to cancel")
}

/// The white cancel fill inside the unchanged Shutter ring.
pub fn cancel_fill() -> impl IntoElement {
    div()
        .size(px(size::SHUTTER_FILL))
        .rounded_full()
        .bg(rgba(color::SHUTTER_FILL))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("icons/x.svg")
                .size(px(size::ICON_LG))
                .text_color(rgba(color::TEXT_ON_SELECTED)),
        )
}
