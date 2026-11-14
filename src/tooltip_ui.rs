//! Control-anchored shortcut tooltips, separate from capture lock explanations.
use std::time::Instant;

use gpui::{
    AnyElement, App, BoxShadow, Context, Div, FontWeight, IntoElement, RenderOnce, Task, Window,
    deferred, div, point, prelude::*, px, relative, rgba,
};

use crate::app_core::tooltips::Hint;
use crate::theme::{color, font, motion, radius, size, space};

/// Which edge of a control the tooltip sits outside.
#[derive(Clone, Copy)]
pub enum Placement {
    Above,
    Below,
}

/// Wrap a control in a stable hover target without replacing its own hover listener.
pub fn control(
    child: impl IntoElement,
    hint: Option<Hint>,
    placement: Placement,
) -> TooltipControl {
    TooltipControl {
        child: child.into_any_element(),
        hint,
        placement,
    }
}

/// A render-once wrapper so callers need no window or tooltip state.
#[derive(IntoElement)]
pub struct TooltipControl {
    child: AnyElement,
    hint: Option<Hint>,
    placement: Placement,
}

/// The pending delay and the visible tooltip's fade clock.
#[derive(Default)]
struct Hover {
    task: Option<Task<()>>,
    shown: Option<Instant>,
}

impl Hover {
    /// Observe resolved shortcuts so keyboard activation dismisses pending and visible hints.
    fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_keystrokes(|hover: &mut Hover, event, _, cx| {
            if event.action.is_some() {
                hover.hide(cx);
            }
        })
        .detach();
        Self::default()
    }

    /// Cancel pending display and remove a visible tooltip immediately.
    fn hide(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.shown = None;
        cx.notify();
    }

    /// Start the design's hover delay; a leave or press cancels the owned task.
    fn enter(&mut self, cx: &mut Context<Self>) {
        self.task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(motion::TOOLTIP_DELAY).await;
            let _ = this.update(cx, |hover, cx| {
                hover.shown = Some(Instant::now());
                cx.notify();
            });
        }));
    }
}

impl RenderOnce for TooltipControl {
    /// Attach cancellation handlers and float the tooltip outside the control's layout.
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(hint) = self.hint else {
            return div().occlude().child(self.child).into_any_element();
        };
        let hover = window.use_keyed_state(
            gpui::SharedString::from(format!("shortcut-tooltip-{}", hint.label)),
            cx,
            |_, cx| Hover::new(cx),
        );
        let shown = hover.read(cx).shown;
        let on_hover = hover.clone();
        div()
            .id(hint.label)
            // Own occlusion here so child buttons cannot mask this hover target.
            .occlude()
            .relative()
            .on_hover(move |inside, _, cx| {
                on_hover.update(cx, |hover, cx| {
                    if *inside {
                        hover.enter(cx);
                    } else {
                        hover.hide(cx);
                    }
                });
            })
            .on_any_mouse_down(move |_, _, cx| hover.update(cx, |hover, cx| hover.hide(cx)))
            .child(self.child)
            .when_some(shown, |target, started| {
                let opacity =
                    (started.elapsed().as_secs_f32() / motion::HOVER.as_secs_f32()).min(1.0);
                if opacity < 1.0 {
                    window.request_animation_frame();
                }
                target.child(deferred(positioned(hint, self.placement, opacity)).with_priority(1))
            })
            .into_any_element()
    }
}

/// Center on a zero-width flex anchor so the tooltip never changes the control's size.
fn positioned(hint: Hint, placement: Placement, opacity: f32) -> Div {
    div()
        .absolute()
        .left(relative(0.5))
        .w_0()
        .flex()
        .justify_center()
        .map(|anchor| match placement {
            Placement::Above => anchor.bottom(relative(1.0)).mb(px(space::S8)),
            Placement::Below => anchor.top(relative(1.0)).mt(px(space::S8)),
        })
        .child(surface(hint).flex_shrink_0().opacity(opacity))
}

/// Render the shared tooltip surface and its optional shortcut keycap.
fn surface(hint: Hint) -> Div {
    div()
        .h(px(size::TOOLTIP))
        .pl(px(space::S12))
        .pr(px(space::S4))
        .flex()
        .items_center()
        .gap(px(space::S8))
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
        .whitespace_nowrap()
        .child(hint.label)
        .children(hint.key.map(keycap))
}

/// Draw a shortcut keycap, widening for names such as Space and Esc.
pub fn keycap(key: &'static str) -> Div {
    div()
        .h(px(size::KEYCAP))
        .min_w(px(size::KEYCAP))
        .px(px(space::S4))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::XS))
        .bg(rgba(color::KEYCAP))
        .border_1()
        .border_color(rgba(color::KEYCAP_BORDER))
        .text_size(px(font::CAPTION))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgba(color::TEXT_SECONDARY))
        .child(key)
}
