//! Shared disabled styling and delayed explanations for capture settings.

use gpui::{
    BoxShadow, Context, Div, FontWeight, IntoElement, Render, Stateful, Window, div, point,
    prelude::*, px, rgba,
};

use crate::app_core::controls::ControlLock;
use crate::theme::{color, font, radius, size, space};

/// Apply the disabled text color and GPUI's standard 500 ms tooltip delay.
pub fn with_lock_explanation(control: Stateful<Div>, lock: ControlLock) -> Stateful<Div> {
    control
        .text_color(rgba(color::TEXT_DISABLED))
        .tooltip(move |_, cx| cx.new(|_| LockTooltip(lock)).into())
}

/// A settings lock explanation with the standard tooltip geometry and typography.
struct LockTooltip(ControlLock);

impl Render for LockTooltip {
    /// Render a 28 px tooltip without a shortcut keycap.
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(size::TOOLTIP))
            .px(px(space::S12))
            .flex()
            .items_center()
            .rounded(px(radius::SM))
            .bg(rgba(color::OVERLAY))
            .border_1()
            .border_color(rgba(color::BORDER_SUBTLE))
            .shadow(vec![BoxShadow {
                color: rgba(color::SHADOW_OVERLAY).into(),
                offset: point(px(0.0), px(8.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
            }])
            .font_family(font::FAMILY)
            .text_size(px(font::SMALL))
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgba(color::TEXT_PRIMARY))
            .child(self.0.message())
    }
}
