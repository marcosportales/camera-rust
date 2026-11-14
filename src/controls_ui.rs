//! Shared surface and mouse observation for the fading controls.
use gpui::{App, Div, IntoElement, MouseMoveEvent, Window, canvas, div, prelude::*};

/// Group absolute controls into one opacity layer without changing their positions.
pub fn layer(opacity: f32) -> Div {
    div().absolute().size_full().opacity(opacity)
}

/// Observe movement even over occluding buttons and menus, without adding a hitbox.
pub fn movement_observer(
    on_move: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase.capture() && bounds.contains(&event.position) {
                    on_move(event, window, cx);
                }
            });
        },
    )
    .absolute()
    .size_full()
}
