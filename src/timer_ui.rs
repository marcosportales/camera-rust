//! Pure visual pieces of the Timer control and its menu.

use gpui::{
    App, BoxShadow, Div, FontWeight, MouseDownEvent, Stateful, Window, div, point, prelude::*, px,
    rgba, svg,
};

use crate::app_core::state::Timer;
use crate::overlays::shadow_control;
use crate::theme::{color, font, radius, size, space};

/// The top-right control row; future Resolution controls follow the Timer with an 8 px gap.
pub fn controls() -> Div {
    div()
        .absolute()
        .top(px(space::S16))
        .right(px(space::S16))
        .flex()
        .gap(px(space::S8))
}

/// Catch any outside press while the menu is open, shielding the controls below it.
pub fn dismiss_layer(on_dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> Div {
    div()
        .absolute()
        .size_full()
        .occlude()
        .on_any_mouse_down(on_dismiss)
}

/// The Timer icon alone when Off, or the icon with its selected value in the set look.
pub fn button(timer: Timer) -> Stateful<Div> {
    let value = match timer {
        Timer::Off => None,
        Timer::ThreeSeconds => Some("3s"),
        Timer::TenSeconds => Some("10s"),
    };
    div()
        .id("timer")
        .occlude()
        .h(px(size::CONTROL))
        .min_w(px(size::CONTROL))
        .when(value.is_some(), |button| {
            button.pl(px(10.0)).pr(px(space::S12))
        })
        .flex()
        .items_center()
        .justify_center()
        .gap(px(space::S4))
        .rounded(px(radius::MD))
        .bg(rgba(if value.is_some() {
            color::CONTROL_ON
        } else {
            color::CONTROL_HOVER
        }))
        .shadow(shadow_control())
        .text_color(rgba(color::TEXT_PRIMARY))
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .child(
            svg()
                .path("icons/timer.svg")
                .size(px(size::ICON_LG))
                .text_color(rgba(color::TEXT_PRIMARY)),
        )
        .children(value)
}

/// The 184 px Timer menu below its button, including its header and shortcut hint.
pub fn menu(opacity: f32) -> Div {
    div()
        .absolute()
        .top(px(size::CONTROL + space::S8))
        .right_0()
        .w(px(184.0))
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
                .child("Timer"),
        )
}

/// A Timer choice, with a check next to the currently selected value.
pub fn row(timer: Timer, selected: Timer) -> Stateful<Div> {
    let (id, label) = match timer {
        Timer::Off => ("timer-off", "Off"),
        Timer::ThreeSeconds => ("timer-3s", "3 seconds"),
        Timer::TenSeconds => ("timer-10s", "10 seconds"),
    };
    div()
        .id(id)
        .h(px(size::MENU_ROW))
        .px(px(space::S12))
        .flex()
        .items_center()
        .justify_between()
        .rounded(px(radius::SM))
        .text_size(px(font::LABEL))
        .font_weight(if timer == selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::MEDIUM
        })
        .hover(|style| style.bg(rgba(color::ITEM_HOVER)))
        .cursor_pointer()
        .child(label)
        .when(timer == selected, |row| {
            row.child(
                svg()
                    .path("icons/check.svg")
                    .size(px(size::ICON_MD))
                    .text_color(rgba(color::TEXT_PRIMARY)),
            )
        })
}

/// The menu's separator and "Cycle timer" hint with the T keycap.
pub fn shortcut_hint() -> Div {
    div()
        .child(
            div()
                .h(px(1.0))
                .my(px(space::S4))
                .bg(rgba(color::ITEM_HOVER)),
        )
        .child(
            div()
                .h(px(size::MENU_ROW))
                .pl(px(space::S12))
                .pr(px(space::S8))
                .flex()
                .items_center()
                .justify_between()
                .text_size(px(font::SMALL))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba(color::TEXT_TERTIARY))
                .child("Cycle timer")
                .child(keycap()),
        )
}

/// The T keycap in the menu footer.
fn keycap() -> Div {
    div()
        .size(px(size::KEYCAP))
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
        .child("T")
}
