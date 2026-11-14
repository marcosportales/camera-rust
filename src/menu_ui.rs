//! Shared floating menu surface used by the Timer and Resolution controls.

use crate::theme::{color, font, radius, size, space};
use gpui::{BoxShadow, Div, FontWeight, div, point, prelude::*, px, rgba};

/// A floating menu below its control, with a header and the overlay shadow.
pub fn surface(title: &'static str, width: f32, opacity: f32) -> Div {
    div()
        .absolute()
        .top(px(size::CONTROL + space::S8))
        .right_0()
        .w(px(width))
        .p(px(space::S4))
        .flex()
        .flex_col()
        .rounded(px(radius::LG))
        .bg(rgba(color::OVERLAY))
        .border_1()
        .border_color(rgba(color::BORDER_SUBTLE))
        .shadow(vec![BoxShadow {
            color: rgba(color::SHADOW_OVERLAY).into(),
            offset: point(px(0.0), px(8.0)),
            blur_radius: px(24.0),
            spread_radius: px(0.0),
        }])
        .text_color(rgba(color::TEXT_PRIMARY))
        .opacity(opacity)
        .occlude()
        .child(
            div()
                .px(px(space::S12))
                .pt(px(space::S8))
                .pb(px(space::S4))
                .text_size(px(font::CAPTION))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(color::TEXT_TERTIARY))
                .child(title),
        )
}
