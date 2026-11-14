//! Shared surface and mouse observation for the fading controls.
use gpui::{App, ClickEvent, Div, IntoElement, MouseMoveEvent, Window, canvas, div, prelude::*};

/// Group absolute controls into one opacity layer without changing their positions.
pub fn layer(opacity: f32) -> Div {
    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .opacity(opacity)
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

/// Compose the fade surface, hidden-control click shield and movement observer.
pub fn surface(
    controls: impl IntoElement,
    opacity: f32,
    hidden: bool,
    on_wake: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_move: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .when(opacity > 0.0, |root| root.child(controls))
        .when(hidden, |root| {
            root.child(
                div()
                    .id("wake-controls")
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .occlude()
                    .on_click(on_wake),
            )
        })
        .child(movement_observer(on_move))
}
