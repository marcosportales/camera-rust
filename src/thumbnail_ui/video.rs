//! Video-specific Thumbnail visuals from the component sheet.

use gpui::{IntoElement, div, prelude::*, px, rgba, svg};

use crate::theme::color;

/// Draw the 18 px play badge three pixels from the bottom-right corner.
pub(super) fn badge() -> impl IntoElement {
    div()
        .absolute()
        .right(px(3.0))
        .bottom(px(3.0))
        .size(px(18.0))
        .rounded_full()
        .bg(rgba(color::THUMB_BADGE))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("icons/play.svg")
                .size(px(10.0))
                .text_color(rgba(0xffffffff)),
        )
}
