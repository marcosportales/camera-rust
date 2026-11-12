//! The Camera window: the Viewfinder filling the window, with the camera name top left.

use std::sync::Arc;

use futures::StreamExt;
use futures::channel::mpsc::{self, Receiver};
use gpui::{
    AsyncWindowContext, Context, FontWeight, IntoElement, ObjectFit, RenderImage, SharedString,
    WeakEntity, Window, div, img, linear_color_stop, linear_gradient, prelude::*, px, rgba, svg,
};

use crate::app_core::backend::CameraBackend;
use crate::app_core::state::{AppState, CameraState, Event};
use crate::gst_camera::{Frame, GstCamera};
use crate::theme::{color, font, size, space};

/// Height of the `scrim_top` gradient behind the top row, in logical px.
const SCRIM_TOP_HEIGHT: f32 = 96.0;

/// Root view: owns the app state, the camera and the latest Viewfinder frame.
pub struct CameraWindow {
    state: AppState,
    /// Kept alive so its pipeline keeps running; dropping it stops the camera.
    _camera: GstCamera,
    frame: Option<Arc<RenderImage>>,
    /// The frame shown before `frame`. Its GPU texture is freed one frame late, because the GPU
    /// may still be drawing it; freeing it at once crashes the Vulkan renderer.
    previous_frame: Option<Arc<RenderImage>>,
}

impl CameraWindow {
    /// Detect the camera, start the Viewfinder and begin showing its frames.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Capacity 0 still buffers one frame per sender: the UI only ever lags one frame behind.
        let (frames_tx, frames_rx) = mpsc::channel(0);
        let mut camera = GstCamera::new(frames_tx);
        let mut state = AppState::new();
        start_camera(&mut camera, &mut state);
        cx.spawn_in(window, async move |this, cx| {
            show_frames(this, frames_rx, cx).await
        })
        .detach();
        Self {
            state,
            _camera: camera,
            frame: None,
            previous_frame: None,
        }
    }

    /// Show `frame` instead of the current one, freeing the texture of the frame before that.
    fn show(&mut self, frame: Frame, window: &mut Window, cx: &mut Context<Self>) {
        let Some(image) = render_image(frame) else {
            return;
        };
        let shown = self.frame.replace(Arc::new(image));
        if let Some(stale) = std::mem::replace(&mut self.previous_frame, shown) {
            cx.drop_image(stale, Some(window));
        }
        cx.notify();
    }
}

/// Detect the camera and start the Viewfinder at the default Resolution, recording the outcome in `state`.
fn start_camera(camera: &mut impl CameraBackend, state: &mut AppState) {
    state.handle(camera.detect_cameras().into());
    let Some(resolution) = state.resolution else {
        return;
    };
    if let Err(error) = camera.start_viewfinder(resolution) {
        state.handle(Event::CameraFailed(error));
    }
}

/// Show each frame the camera sends until the window or the camera goes away.
async fn show_frames(
    this: WeakEntity<CameraWindow>,
    mut frames: Receiver<Frame>,
    cx: &mut AsyncWindowContext,
) {
    while let Some(frame) = frames.next().await {
        if this
            .update_in(cx, |view, window, cx| view.show(frame, window, cx))
            .is_err()
        {
            break;
        }
    }
}

/// Wrap a BGRA frame as a GPUI image (GPUI's `RenderImage` stores BGRA bytes).
fn render_image(frame: Frame) -> Option<RenderImage> {
    let buffer = image::RgbaImage::from_raw(frame.size.width, frame.size.height, frame.bgra)?;
    Some(RenderImage::new([image::Frame::new(buffer)]))
}

impl Render for CameraWindow {
    /// Draw the Viewfinder and camera name when the camera is ready, else only the app background.
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let root = div()
            .size_full()
            .relative()
            .bg(rgba(color::APP_BG))
            .font_family(font::FAMILY);
        let CameraState::Ready(camera) = &self.state.camera else {
            return root;
        };
        root.bg(rgba(color::LETTERBOX))
            .children(self.frame.clone().map(viewfinder_image))
            .child(scrim_top())
            .child(camera_label(SharedString::from(camera.name.clone())))
    }
}

/// The latest frame, as large as fits the window without distortion, centered.
fn viewfinder_image(frame: Arc<RenderImage>) -> impl IntoElement {
    img(frame).size_full().object_fit(ObjectFit::Contain)
}

/// Gradient from `color::SCRIM_TOP` at the top edge to transparent, so the top row stays legible.
fn scrim_top() -> impl IntoElement {
    // Same color with the alpha byte (lowest, in 0xRRGGBBAA) cleared, so only opacity fades.
    let transparent = color::SCRIM_TOP & 0xFFFF_FF00;
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .h(px(SCRIM_TOP_HEIGHT))
        .bg(linear_gradient(
            180.0,
            linear_color_stop(rgba(color::SCRIM_TOP), 0.0),
            linear_color_stop(rgba(transparent), 1.0),
        ))
}

/// The `camera` icon and the camera name, top left.
fn camera_label(name: SharedString) -> impl IntoElement {
    let text_color = rgba(color::TEXT_SECONDARY);
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
        .text_color(text_color)
        .child(
            svg()
                .path("icons/camera.svg")
                .size(px(size::ICON_SM))
                .text_color(text_color),
        )
        .child(name)
}
