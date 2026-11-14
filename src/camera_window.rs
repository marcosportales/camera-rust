//! The Camera window: the Viewfinder filling the window, with the camera name top left.

mod auto_hide;
mod countdown;
mod resolution;
mod settings;
mod shutter;
mod thumbnail;

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{self, Receiver};
use gpui::{
    AsyncWindowContext, Context, Div, FocusHandle, IntoElement, RenderImage, SharedString, Task,
    WeakEntity, Window, actions, div, prelude::*, px, rgba,
};

use crate::app_core::backend::CameraBackend;
use crate::app_core::state::{AppState, CameraState, Event, Mode, VideoCapture};
use crate::feedback::{CaptureFeedback, MenuFade, ModeTransition};
use crate::gst_camera::{Frame, GstCamera};
use crate::gst_recording::{DefaultMicrophone, H264AacMp4, Recorder};
use crate::mode_pill;
use crate::overlays::{camera_label, viewfinder_image};
use crate::photo_ui::{self, FillTransition};
use crate::recording_ui;
use crate::theme::{color, font, size, space};

actions!(
    camera,
    [
        PressShutter,
        SelectPhoto,
        SelectVideo,
        CycleTimer,
        CloseTimerMenu,
        ToggleMirror
    ]
);

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
    timer_menu_fade: MenuFade,
    resolution_menu_fade: MenuFade,
    shutter_hovered: bool,
    shutter_pressed: bool,
    shutter_fill: FillTransition,
    /// Advances the elapsed time while a Video records; dropping it stops the ticking.
    recording_clock: Option<Task<()>>,
    countdown_clock: countdown::CountdownClock,
    controls_visibility: auto_hide::ControlsVisibility,
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
        let mut camera = GstCamera::new(frames_tx, Recorder::new(H264AacMp4, DefaultMicrophone));
        let mut state = AppState::new();
        if let Some(path) = crate::capture_store::newest_capture(
            &crate::capture_store::photos_folder(),
            &crate::capture_store::videos_folder(),
        ) {
            state.handle(Event::CaptureFound(path));
        }
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
            timer_menu_fade: MenuFade::new(),
            resolution_menu_fade: MenuFade::new(),
            shutter_hovered: false,
            shutter_pressed: false,
            shutter_fill: FillTransition::new(),
            recording_clock: None,
            countdown_clock: countdown::CountdownClock::new(),
            controls_visibility: auto_hide::ControlsVisibility::new(),
            frame: None,
            previous_frame: None,
        }
    }

    /// Show `frame` instead of the current one, freeing the texture of the frame before that.
    fn show(&mut self, frame: Frame, window: &mut Window, cx: &mut Context<Self>) {
        let Some(image) = crate::viewfinder::render_image(frame, self.state.mirror) else {
            return;
        };
        self.replace_frame(image, window, cx);
    }

    /// Replace the displayed image and retire its texture one frame late.
    fn replace_frame(&mut self, image: RenderImage, window: &mut Window, cx: &mut Context<Self>) {
        let shown = self.frame.replace(Arc::new(image));
        if let Some(stale) = std::mem::replace(&mut self.previous_frame, shown) {
            cx.drop_image(stale, Some(window));
        }
        cx.notify();
    }

    /// Apply the shared Mirror event and reverse the current displayed frame if it changed.
    fn toggle_mirror(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let was_mirrored = self.state.mirror;
        self.state.handle(Event::ToggleMirror);
        if self.state.mirror != was_mirrored {
            if let Some(image) = self
                .frame
                .as_deref()
                .and_then(crate::viewfinder::flipped_image)
            {
                self.replace_frame(image, window, cx);
            }
            cx.notify();
        }
    }

    /// Switch to `mode`; does nothing if it is already active, the camera is not ready, or a Video records.
    fn select_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        let previous = self.state.mode;
        if !matches!(self.state.camera, CameraState::Ready(_)) {
            return;
        }
        self.state.handle(Event::SetMode(mode));
        if self.state.mode == previous {
            return;
        }
        self.mode_transition.switched_from(previous);
        self.update_shutter_fill(cx);
        cx.notify();
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
    state.retry_camera(camera);
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

impl CameraWindow {
    /// The root surface and keyboard actions, scoped to the Camera window's focus.
    fn root(&self, cx: &mut Context<Self>) -> Div {
        div()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &PressShutter, _, cx| this.press_shutter(cx)))
            .on_action(
                cx.listener(|this, _: &SelectPhoto, _, cx| this.select_mode(Mode::Photo, cx)),
            )
            .on_action(
                cx.listener(|this, _: &SelectVideo, _, cx| this.select_mode(Mode::Video, cx)),
            )
            .on_action(cx.listener(|this, _: &CycleTimer, _, cx| {
                this.settings_event(Event::CycleTimer, cx);
            }))
            .on_action(cx.listener(|this, _: &CloseTimerMenu, _, cx| {
                this.cancel_countdown(cx);
                this.settings_event(Event::CloseTimerMenu, cx);
                this.settings_event(Event::CloseResolutionMenu, cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleMirror, window, cx| {
                this.toggle_mirror(window, cx);
            }))
            .size_full()
            .relative()
            .bg(rgba(color::APP_BG))
            .font_family(font::FAMILY)
    }
}

impl Render for CameraWindow {
    /// Draw the Viewfinder and camera name when the camera is ready, else only the app background.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_controls(false, cx);
        let root = self.root(cx);
        let CameraState::Ready(camera) = &self.state.camera else {
            return root.children(crate::error_ui::screen(
                &self.state,
                cx.listener(|this, _, _, cx| {
                    this.state.retry_camera(&mut this.camera);
                    cx.notify();
                }),
            ));
        };
        let root = root
            .bg(rgba(color::LETTERBOX))
            .children(self.frame.clone().map(viewfinder_image))
            // The flash sits under the scrims and controls.
            .children(self.feedback.flash.map(photo_ui::flash))
            .child(crate::countdown_ui::backdrop(
                self.state.countdown_numeral().is_some(),
            ))
            .child(self.overlay_controls(window, cx))
            .child(camera_label(SharedString::from(camera.name.clone())));
        if let Some(numeral) = self.state.countdown_numeral() {
            return root.child(crate::countdown_ui::countdown(numeral));
        }
        match self.state.video {
            Some(VideoCapture::Recording(recording)) => {
                root.child(recording_ui::timecode_pill(recording))
            }
            Some(VideoCapture::Saving) => root.child(recording_ui::saving_label()),
            None => root.children(
                self.feedback
                    .toast
                    .zip(self.state.last_capture.as_ref())
                    .map(|(id, outcome)| photo_ui::toast(id, outcome)),
            ),
        }
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
        crate::tooltip_ui::control(
            mode_pill::segment(
                target,
                self.state.mode,
                self.mode_transition.previous,
                self.mode_transition.switches,
                self.state.control_lock(),
                cx.listener(move |this, _, _, cx| this.select_mode(target, cx)),
            ),
            self.shortcut_hint(match target {
                Mode::Photo => crate::app_core::tooltips::Control::Photo,
                Mode::Video => crate::app_core::tooltips::Control::Video,
            }),
            crate::tooltip_ui::Placement::Above,
        )
    }
}
