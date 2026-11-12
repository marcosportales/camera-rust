//! The Camera window: the Viewfinder filling the window, with the camera name top left.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{self, Receiver};
use gpui::{
    AsyncWindowContext, Context, FocusHandle, FontWeight, IntoElement, MouseButton, ObjectFit,
    RenderImage, SharedString, WeakEntity, Window, actions, div, img, linear_color_stop,
    linear_gradient, prelude::*, px, rgba, svg,
};

use crate::app_core::backend::CameraBackend;
use crate::app_core::state::{AppState, CameraState, Event, PhotoOutcome};
use crate::gst_camera::{Frame, GstCamera};
use crate::photo_store::photos_folder;
use crate::photo_ui::{self, FLASH_DURATION, FillTransition, ShutterFill, TOAST_DURATION};
use crate::theme::{color, font, size, space};

actions!(camera, [TakePhoto]);

/// Key context of the root view, used to scope key bindings.
pub const KEY_CONTEXT: &str = "Camera";

/// Height of the `scrim_top` gradient behind the top row, in logical px.
const SCRIM_TOP_HEIGHT: f32 = 96.0;
/// Height of the `scrim_bottom` gradient behind the bottom controls, in logical px.
const SCRIM_BOTTOM_HEIGHT: f32 = 200.0;

/// Root view: owns the app state, the camera and the latest Viewfinder frame.
pub struct CameraWindow {
    state: AppState,
    /// Owned so its pipeline keeps running; dropping it stops the camera.
    camera: GstCamera,
    focus: FocusHandle,
    /// Id of the flash being played, if any. Ids grow with every Photo, so each one restarts the animation.
    flash: Option<u64>,
    /// Id of the toast being shown, if any; its text comes from `state.photo`.
    toast: Option<u64>,
    photos_taken: u64,
    shutter_hovered: bool,
    shutter_pressed: bool,
    shutter_fill: FillTransition,
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
        let focus = cx.focus_handle();
        window.focus(&focus);
        Self {
            state,
            camera,
            focus,
            flash: None,
            toast: None,
            photos_taken: 0,
            shutter_hovered: false,
            shutter_pressed: false,
            shutter_fill: FillTransition::new(),
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

    /// Take a Photo; once it is written, flash and toast the outcome. A failure only toasts.
    fn take_photo(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.state.camera, CameraState::Ready(_)) {
            return;
        }
        self.state.take_photo(&mut self.camera, &photos_folder());
        self.photos_taken += 1;
        let id = self.photos_taken;
        if matches!(self.state.photo, Some(PhotoOutcome::Saved(_))) {
            self.flash = Some(id);
            after(cx, FLASH_DURATION, move |view, cx| view.end_flash(id, cx));
        }
        self.toast = Some(id);
        after(cx, TOAST_DURATION, move |view, cx| {
            if view.toast == Some(id) {
                view.toast = None;
                cx.notify();
            }
        });
        self.update_shutter_fill(cx);
    }

    /// Remove flash `id`, unless a newer Photo has replaced it.
    fn end_flash(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.flash == Some(id) {
            self.flash = None;
            self.update_shutter_fill(cx);
        }
    }

    /// The mouse button is no longer held on the Shutter.
    fn release_shutter(&mut self, cx: &mut Context<Self>) {
        self.shutter_pressed = false;
        self.update_shutter_fill(cx);
    }

    /// Fade the Shutter fill to the look that matches the flash, press and hover state.
    fn update_shutter_fill(&mut self, cx: &mut Context<Self>) {
        let target = if self.flash.is_some() || self.shutter_pressed {
            ShutterFill::Pressed
        } else if self.shutter_hovered {
            ShutterFill::Hover
        } else {
            ShutterFill::Rest
        };
        if self.shutter_fill.retarget(target) {
            cx.notify();
        }
    }
}

/// Run `action` on the window once `delay` has passed.
fn after(
    cx: &mut Context<CameraWindow>,
    delay: Duration,
    action: impl FnOnce(&mut CameraWindow, &mut Context<CameraWindow>) + 'static,
) {
    cx.spawn(async move |this, cx| {
        cx.background_executor().timer(delay).await;
        let _ = this.update(cx, action);
    })
    .detach();
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &TakePhoto, _, cx| this.take_photo(cx)))
            .size_full()
            .relative()
            .bg(rgba(color::APP_BG))
            .font_family(font::FAMILY);
        let CameraState::Ready(camera) = &self.state.camera else {
            return root;
        };
        root.bg(rgba(color::LETTERBOX))
            .children(self.frame.clone().map(viewfinder_image))
            // The flash sits under the scrims and controls.
            .children(self.flash.map(photo_ui::flash))
            .child(scrim_top())
            .child(scrim_bottom())
            .child(camera_label(SharedString::from(camera.name.clone())))
            .child(self.shutter(cx))
            .children(
                self.toast
                    .zip(self.state.photo.as_ref())
                    .map(|(id, outcome)| photo_ui::toast(id, outcome)),
            )
    }
}

impl CameraWindow {
    /// The Shutter at the bottom center; clicking it takes a Photo.
    fn shutter(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button = photo_ui::shutter(self.shutter_fill)
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.shutter_hovered = *hovered;
                this.update_shutter_fill(cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.shutter_pressed = true;
                    this.update_shutter_fill(cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.release_shutter(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.release_shutter(cx)),
            )
            .on_click(cx.listener(|this, _, _, cx| this.take_photo(cx)));
        div()
            .absolute()
            .bottom(px(space::S24))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(button)
    }
}

/// The latest frame, as large as fits the window without distortion, centered.
fn viewfinder_image(frame: Arc<RenderImage>) -> impl IntoElement {
    img(frame).size_full().object_fit(ObjectFit::Contain)
}

/// Gradient from `color::SCRIM_TOP` at the top edge to transparent, so the top row stays legible.
fn scrim_top() -> impl IntoElement {
    scrim(color::SCRIM_TOP, SCRIM_TOP_HEIGHT, 180.0).top_0()
}

/// Gradient from `color::SCRIM_BOTTOM` at the bottom edge to transparent, so the bottom controls stay legible.
fn scrim_bottom() -> impl IntoElement {
    scrim(color::SCRIM_BOTTOM, SCRIM_BOTTOM_HEIGHT, 0.0).bottom_0()
}

/// A full-width band `height` tall fading from `edge_color` to transparent, along `angle` degrees.
fn scrim(edge_color: u32, height: f32, angle: f32) -> gpui::Div {
    // Same color with the alpha byte (lowest, in 0xRRGGBBAA) cleared, so only opacity fades.
    let transparent = edge_color & 0xFFFF_FF00;
    div()
        .absolute()
        .left_0()
        .right_0()
        .h(px(height))
        .bg(linear_gradient(
            angle,
            linear_color_stop(rgba(edge_color), 0.0),
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
