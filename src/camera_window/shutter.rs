//! The Shutter of the Camera window: what pressing it does and how it looks.

use std::time::Duration;

use gpui::{Context, IntoElement, MouseButton, div, prelude::*, px};

use super::{CameraWindow, after};
use crate::app_core::state::{Event, PhotoOutcome, ShutterAction};
use crate::capture_store::{photos_folder, videos_folder};
use crate::photo_ui::{self, FLASH_DURATION, ShutterFill, ShutterLook, TOAST_DURATION};
use crate::recording_ui;
use crate::theme::space;

/// How often the elapsed time of a Video advances.
const TICK: Duration = Duration::from_secs(1);

impl CameraWindow {
    /// Take a Photo, or start or stop a Video, depending on the Mode and whether a Video is recording.
    pub(super) fn press_shutter(&mut self, cx: &mut Context<Self>) {
        match self.state.shutter_action() {
            Some(ShutterAction::TakePhoto) => self.take_photo(cx),
            Some(ShutterAction::StartRecording) => self.start_recording(cx),
            Some(ShutterAction::StopRecording) => self.stop_recording(cx),
            None => {}
        }
    }

    /// Take a Photo; once it is written, flash and toast the outcome. A failure only toasts.
    fn take_photo(&mut self, cx: &mut Context<Self>) {
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

    /// Start a Video and advance its elapsed time every second until it stops.
    fn start_recording(&mut self, cx: &mut Context<Self>) {
        self.state
            .start_recording(&mut self.camera, &videos_folder());
        if self.state.recording.is_some() {
            self.recording_clock = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    let ticked = this.update(cx, |view, cx| {
                        view.state.handle(Event::RecordingTick);
                        cx.notify();
                    });
                    if ticked.is_err() {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }

    /// Stop the Video and its ticking; the Shutter returns to its Video Mode look.
    fn stop_recording(&mut self, cx: &mut Context<Self>) {
        self.state.stop_recording(&mut self.camera);
        self.recording_clock = None;
        cx.notify();
    }

    /// Remove flash `id`, unless a newer Photo has replaced it.
    pub(super) fn end_flash(&mut self, id: u64, cx: &mut Context<Self>) {
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
    pub(super) fn update_shutter_fill(&mut self, cx: &mut Context<Self>) {
        let fill = if self.feedback.flash.is_some() || self.shutter_pressed {
            ShutterFill::Pressed
        } else if self.shutter_hovered {
            ShutterFill::Hover
        } else {
            ShutterFill::Rest
        };
        let mode = self.state.mode;
        // Also redraws the stop square, whose pressed color is not part of the fill transition.
        self.shutter_fill.retarget(ShutterLook { mode, fill });
        cx.notify();
    }

    /// The Shutter at the bottom center: a stop square while recording, else the Mode's fill.
    pub(super) fn shutter(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button = match self.state.recording {
            Some(_) => photo_ui::shutter(recording_ui::stop_square(self.shutter_pressed)),
            None => photo_ui::shutter(photo_ui::shutter_fill(self.shutter_fill)),
        };
        let button = button
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
            .on_click(cx.listener(|this, _, _, cx| this.press_shutter(cx)));
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
