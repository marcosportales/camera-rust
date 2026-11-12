//! The Camera window: the Viewfinder filling the window, with the camera name top left.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{self, Receiver};
use gpui::{
    AsyncWindowContext, Context, FocusHandle, IntoElement, MouseButton, RenderImage, SharedString,
    WeakEntity, Window, actions, div, prelude::*, px, rgba,
};

use crate::app_core::backend::CameraBackend;
use crate::app_core::state::{AppState, CameraState, Event, Mode, PhotoOutcome};
use crate::feedback::{CaptureFeedback, ModeTransition};
use crate::gst_camera::{Frame, GstCamera};
use crate::mode_pill;
use crate::overlays::{camera_label, scrim_bottom, scrim_top, viewfinder_image};
use crate::photo_store::photos_folder;
use crate::photo_ui::{
    self, FLASH_DURATION, FillTransition, ShutterFill, ShutterLook, TOAST_DURATION,
};
use crate::theme::{color, font, size, space};

actions!(camera, [TakePhoto, SelectPhoto, SelectVideo]);

/// Key context of the root view, used to scope key bindings.
pub const KEY_CONTEXT: &str = "Camera";

/// Root view: owns the app state, the camera and the latest Viewfinder frame.
pub struct CameraWindow {
    state: AppState,
    /// Owned so its pipeline keeps running; dropping it stops the camera.
    camera: GstCamera,
    focus: FocusHandle,
    feedback: CaptureFeedback,
    mode_transition: ModeTransition,
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
            feedback: CaptureFeedback::default(),
            mode_transition: ModeTransition::new(),
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
        // Recording arrives in a later ticket; in Video Mode the Shutter does nothing.
        if !matches!(self.state.camera, CameraState::Ready(_)) || self.state.mode != Mode::Photo {
            return;
        }
        self.state.take_photo(&mut self.camera, &photos_folder());
        let saved = matches!(self.state.photo, Some(PhotoOutcome::Saved(_)));
        let id = self.feedback.begin(saved);
        if saved {
            after(cx, FLASH_DURATION, move |view, cx| view.end_flash(id, cx));
        }
        after(cx, TOAST_DURATION, move |view, cx| {
            if view.feedback.end_toast(id) {
                cx.notify();
            }
        });
        self.update_shutter_fill(cx);
    }

    /// Switch to `mode`; does nothing if it is already active or the camera is not ready.
    fn select_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if !matches!(self.state.camera, CameraState::Ready(_)) || self.state.mode == mode {
            return;
        }
        self.mode_transition.switched_from(self.state.mode);
        self.state.handle(Event::SetMode(mode));
        self.update_shutter_fill(cx);
        cx.notify();
    }

    /// Remove flash `id`, unless a newer Photo has replaced it.
    fn end_flash(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.feedback.end_flash(id) {
            self.update_shutter_fill(cx);
        }
    }

    /// The mouse button is no longer held on the Shutter.
    fn release_shutter(&mut self, cx: &mut Context<Self>) {
        self.shutter_pressed = false;
        self.update_shutter_fill(cx);
    }

    /// Fade the Shutter fill to the look that matches the Mode and the flash, press and hover state.
    fn update_shutter_fill(&mut self, cx: &mut Context<Self>) {
        let fill = if self.feedback.flash.is_some() || self.shutter_pressed {
            ShutterFill::Pressed
        } else if self.shutter_hovered {
            ShutterFill::Hover
        } else {
            ShutterFill::Rest
        };
        let mode = self.state.mode;
        if self.shutter_fill.retarget(ShutterLook { mode, fill }) {
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
            .on_action(
                cx.listener(|this, _: &SelectPhoto, _, cx| this.select_mode(Mode::Photo, cx)),
            )
            .on_action(
                cx.listener(|this, _: &SelectVideo, _, cx| this.select_mode(Mode::Video, cx)),
            )
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
            .children(self.feedback.flash.map(photo_ui::flash))
            .child(scrim_top())
            .child(scrim_bottom())
            .child(camera_label(SharedString::from(camera.name.clone())))
            .child(self.mode_pill(cx))
            .child(self.shutter(cx))
            .children(
                self.feedback
                    .toast
                    .zip(self.state.photo.as_ref())
                    .map(|(id, outcome)| photo_ui::toast(id, outcome)),
            )
    }
}

impl CameraWindow {
    /// The Mode pill centered above the Shutter, `space.16` above its top edge.
    fn mode_pill(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .bottom(px(space::S24 + size::SHUTTER + space::S16))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                mode_pill::track()
                    .child(self.mode_segment(Mode::Photo, cx))
                    .child(self.mode_segment(Mode::Video, cx)),
            )
    }

    /// The pill segment that selects `target` when clicked.
    fn mode_segment(&self, target: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        mode_pill::segment(
            target,
            self.state.mode,
            self.mode_transition.previous,
            self.mode_transition.switches,
            cx.listener(move |this, _, _, cx| this.select_mode(target, cx)),
        )
    }

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
