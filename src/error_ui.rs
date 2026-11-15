//! Camera failure screens and their Retry action.
use crate::app_core::{
    backend::CameraError,
    state::{AppState, CameraState},
};
use crate::theme::{color, font, radius, size, space};
use gpui::{App, ClickEvent, Div, FontWeight, Window, div, prelude::*, px, rgba, svg};

/// Show the classified failure with its device label and a retry button.
pub fn screen(
    state: &AppState,
    retry: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Option<Div> {
    let CameraState::Error(error) = state.camera else {
        return None;
    };
    let name = if error == CameraError::NoCamera {
        "No camera"
    } else {
        state.last_camera_name.as_deref().unwrap_or("Camera")
    };
    let (title, message) = match error {
        CameraError::InUse => ("Camera is in use", format!("Another app is using {name}. Close it, then try again.")),
        CameraError::AccessDenied => ("Camera access denied", "Camera doesn’t have permission to open the webcam. Check the device permissions, then try again.".into()),
        CameraError::NoCamera => ("No camera found", "Connect a webcam, then try again.".into()),
        CameraError::Other => ("Unable to start camera", "Something went wrong while opening the camera. Try again. If the problem continues, check that the camera software and required media components are installed.".into()),
    };
    Some(
        div()
            .size_full()
            .relative()
            .child(device_label(name))
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(space::S24))
                    .px(px(space::S24))
                    .child(error_tile(error))
                    .child(error_text(title, message))
                    .child(retry_button(retry)),
            ),
    )
}

/// Label the camera involved in the error.
fn device_label(name: &str) -> Div {
    div()
        .absolute()
        .top(px(space::S16))
        .left(px(space::S16))
        .h(px(size::CONTROL))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgba(color::TEXT_TERTIARY))
        .child(
            svg()
                .path("icons/camera.svg")
                .size(px(size::ICON_SM))
                .text_color(rgba(color::TEXT_TERTIARY)),
        )
        .child(name.to_owned())
}

/// Show an alert for general failures and a camera-off icon for known camera errors.
fn error_tile(error: CameraError) -> Div {
    div()
        .size(px(56.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::FULL))
        .bg(rgba(color::ICON_TILE))
        .border_1()
        .border_color(rgba(color::BORDER_SUBTLE))
        .child(
            svg()
                .path(if error == CameraError::Other {
                    "icons/triangle-alert.svg"
                } else {
                    "icons/camera-off.svg"
                })
                .size(px(size::ICON_XL))
                .text_color(rgba(color::TEXT_SECONDARY)),
        )
}

/// Center the error title and its explanatory message.
fn error_text(title: &'static str, message: String) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(space::S8))
        .max_w(px(380.))
        .text_center()
        .child(
            div()
                .text_size(px(font::TITLE))
                .line_height(px(24.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(color::TEXT_PRIMARY))
                .child(title),
        )
        .child(
            div()
                .text_size(px(font::BODY))
                .line_height(px(20.))
                .text_color(rgba(color::TEXT_SECONDARY))
                .child(message),
        )
}

/// Primary action to detect the camera again.
fn retry_button(retry: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id("retry-camera")
        .h(px(36.))
        .pl(px(14.))
        .pr(px(18.))
        .rounded(px(radius::FULL))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .bg(rgba(color::SELECTED))
        .text_color(rgba(color::TEXT_ON_SELECTED))
        .text_size(px(font::LABEL))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .on_click(retry)
        .child(
            svg()
                .path("icons/rotate-cw.svg")
                .size(px(size::ICON_MD))
                .text_color(rgba(color::TEXT_ON_SELECTED)),
        )
        .child("Retry")
}
